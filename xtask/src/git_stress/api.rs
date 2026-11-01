use super::{Config, compose_up};
use anyhow::{Context, Result, bail};
use reqwest::blocking::{Client, Response};
use serde_json::{Value, json};
use std::{path::Path, thread, time::Duration};

#[path = "api/cleanup.rs"]
mod cleanup;
pub use cleanup::cleanup;

pub struct Repository {
    pub url: String,
    pub token: String,
    pub cleanup: Option<CleanupResources>,
}

pub struct CleanupResources {
    base_url: String,
    api_token: String,
    user_id: String,
    organization_id: String,
    project_id: String,
    project_path: String,
    access_token_id: String,
}

pub fn prepare(root: &Path, config: &Config, run_id: &str) -> Result<Repository> {
    if config.compose {
        compose_up(root, &config.project, port(&config.base_url), config.build)?;
        wait_for_server(&config.base_url, &config.project, root)?;
    }
    if !config.project_url.is_empty() {
        let token = std::env::var(&config.token_env)
            .with_context(|| format!("set {} for existing repository mode", config.token_env))?;
        return Ok(Repository {
            url: clean_url(&config.project_url)?,
            token,
            cleanup: None,
        });
    }

    let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
    get_ok(&client, &format!("{}/api/health", config.base_url), None)?;
    let session = request(
        &client,
        &config.base_url,
        "POST",
        "/auth/login",
        None,
        Some(json!({"username": format!("{run_id}-owner"), "password": "password"})),
    )?;
    let access_token = field(&session, &["token"]);
    let user_id = number(&session, &["user_id", "id"]);
    if access_token.is_empty() || user_id <= 0 {
        bail!("login response omitted token or user id");
    }

    let organizations = request(
        &client,
        &config.base_url,
        "GET",
        "/orgs",
        Some(&access_token),
        None,
    )?;
    let mut organization = array(&organizations).into_iter().next();
    if organization.is_none() {
        organization = Some(request(
            &client,
            &config.base_url,
            "POST",
            "/orgs",
            Some(&access_token),
            Some(json!({
                "key": format!("{run_id}-org"), "path_key": format!("{run_id}-org"),
                "name": format!("Git Perf {run_id}"),
                "description": "Created by Rust git client stress setup", "visibility": "private"
            })),
        )?);
    }
    let organization_id = field(organization.as_ref().unwrap(), &["id"]);
    if organization_id.is_empty() {
        bail!("organization response omitted id");
    }
    let organization_id_number = organization_id
        .parse::<i64>()
        .context("organization ID is not a decimal i64")?;
    let project = request(
        &client,
        &config.base_url,
        "POST",
        "/projects",
        Some(&access_token),
        Some(json!({
                "organization_id": organization_id_number, "key": format!("{run_id}-project"),
            "path_key": format!("{run_id}-project"), "name": format!("Git Perf {run_id}"),
            "description": "Project created by Git client stress setup", "visibility": "private", "default_branch": "main"
        })),
    )?;
    let project_id = field(&project, &["id"]);
    let project_path = field(&project, &["full_path"]);
    let clone_url = field(&project, &["clone_http_url"]);
    let clone_url = if clone_url.is_empty() {
        format!(
            "{}/{}.git",
            config.base_url,
            field(&project, &["full_path"])
        )
    } else {
        clone_url
    };
    if project_id.is_empty() || project_path.is_empty() || clone_url.ends_with("/.git") {
        bail!("project response omitted id or clone URL");
    }
    let credential = request(
        &client,
        &config.base_url,
        "POST",
        &format!("/projects/{project_id}/access-tokens"),
        Some(&access_token),
        Some(json!({
            "name": format!("{run_id}-git-token"), "username": config.token_username,
            "scopes": ["read_repository", "write_repository"], "created_by_user_id": user_id, "expires_at": ""
        })),
    )?;
    let token = field(&credential, &["token"]);
    let access_token_id = credential
        .get("project_token")
        .map(|value| field(value, &["id"]))
        .unwrap_or_default();
    if token.is_empty() || access_token_id.is_empty() {
        bail!("project access-token response omitted token or token ID");
    }
    Ok(Repository {
        url: clean_url(&clone_url)?,
        token,
        cleanup: Some(CleanupResources {
            base_url: config.base_url.clone(),
            api_token: access_token,
            user_id: user_id.to_string(),
            organization_id,
            project_id,
            project_path,
            access_token_id,
        }),
    })
}

pub fn request(
    client: &Client,
    base: &str,
    method: &str,
    route: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> Result<Value> {
    let url = format!("{}/api/v1{}", base.trim_end_matches('/'), route);
    let mut builder = client.request(method.parse()?, &url);
    if let Some(value) = token {
        builder = builder.bearer_auth(value);
    }
    if let Some(value) = body {
        builder = builder.json(&value);
    }
    let response = builder
        .send()
        .with_context(|| format!("{method} {route} request failed"))?;
    response_value(response, method, route)
}

fn response_value(response: Response, method: &str, route: &str) -> Result<Value> {
    let status = response.status();
    if !status.is_success() {
        bail!("{method} {route} returned HTTP {status}");
    }
    let value: Value = response
        .json()
        .with_context(|| format!("{method} {route} returned invalid JSON"))?;
    Ok(unwrap(value))
}

fn get_ok(client: &Client, url: &str, token: Option<&str>) -> Result<()> {
    let mut request = client.get(url);
    if let Some(value) = token {
        request = request.bearer_auth(value);
    }
    let response = request.send()?;
    if !response.status().is_success() {
        bail!("health check returned HTTP {}", response.status());
    }
    Ok(())
}

fn wait_for_server(base: &str, project: &str, root: &Path) -> Result<()> {
    let client = Client::builder().timeout(Duration::from_secs(3)).build()?;
    let mut last_error = String::new();
    for _ in 0..60 {
        let health = get_ok(
            &client,
            &format!("{}/api/health", base.trim_end_matches('/')),
            None,
        );
        let home = get_ok(&client, &format!("{}/", base.trim_end_matches('/')), None);
        if health.is_ok() && home.is_ok() {
            return Ok(());
        }
        last_error = health
            .err()
            .or_else(|| home.err())
            .map(|e| e.to_string())
            .unwrap_or_default();
        thread::sleep(Duration::from_secs(2));
    }
    let _ = std::process::Command::new("docker")
        .current_dir(root)
        .args([
            "compose",
            "-p",
            project,
            "-f",
            "docker-compose.blackbox.yaml",
            "logs",
        ])
        .status();
    bail!("blackbox server did not become healthy: {last_error}")
}

fn clean_url(raw: &str) -> Result<String> {
    let mut url = url::Url::parse(raw).context("invalid project URL")?;
    url.set_username("")
        .map_err(|_| anyhow::anyhow!("invalid URL username"))?;
    url.set_password(None)
        .map_err(|_| anyhow::anyhow!("invalid URL password"))?;
    url.set_query(None);
    url.set_fragment(None);
    Ok(url.to_string())
}

pub fn unwrap(mut value: Value) -> Value {
    for _ in 0..4 {
        if let Some(object) = value.as_object() {
            if (object.contains_key("code") || object.contains_key("message"))
                && object.contains_key("data")
            {
                value = object["data"].clone();
                continue;
            }
            if let Some(body) = object.get("body").or_else(|| object.get("Body")) {
                value = body.clone();
                continue;
            }
        }
        break;
    }
    value
}

pub fn array(value: &Value) -> Vec<Value> {
    if let Some(values) = value.as_array() {
        return values.clone();
    }
    ["items", "organizations", "projects"]
        .iter()
        .find_map(|key| value.get(key)?.as_array().cloned())
        .unwrap_or_default()
}
pub fn field(value: &Value, names: &[&str]) -> String {
    names
        .iter()
        .find_map(|name| {
            value.get(*name).filter(|v| !v.is_null()).map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| v.to_string())
            })
        })
        .unwrap_or_default()
}
pub fn number(value: &Value, names: &[&str]) -> i64 {
    field(value, names).parse().unwrap_or(0)
}
fn port(base: &str) -> u16 {
    url::Url::parse(base)
        .ok()
        .and_then(|v| v.port())
        .unwrap_or(18080)
}
