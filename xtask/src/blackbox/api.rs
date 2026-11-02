use super::{BlackboxArgs, Reporter, RunConfig, grant_admin};
use anyhow::{Context, Result, bail};
use reqwest::{Method, StatusCode, blocking::Client};
use serde_json::{Value, json};
use std::path::Path;
use uuid::Uuid;

mod distributed;
mod http;
mod packages;
mod web;
pub(super) use distributed::distributed;
pub(super) use http::{data, id, request_json, require_denied, require_status};

pub struct Scenario {
  pub owner_username: String,
  pub owner_token: String,
  pub outsider_token: String,
  pub project_id: String,
  pub project_path: String,
  pub outsider_project_path: String,
  pub suffix: String,
  pub git_credential: Option<super::git::GitCredential>,
}

pub fn run(
  root: &Path,
  args: &BlackboxArgs,
  config: &RunConfig,
  reporter: &mut Reporter,
) -> Result<Scenario> {
  let client = Client::builder()
    .timeout(std::time::Duration::from_secs(120))
    .build()?;
  let health = client
    .get(format!("{}/api/health", config.http_url))
    .send()
    .context("request HTTP health endpoint")?;
  require_status(health.status(), StatusCode::OK, "HTTP health")?;
  reporter.pass("http-health", "HTTP health returned 200");

  web::run(root, config, reporter)?;

  for path in [
    "/docs",
    "/docs/",
    "/docs/swagger-ui-bundle.js",
    "/openapi.json",
  ] {
    let response = client.get(format!("{}{path}", config.http_url)).send()?;
    require_status(
      response.status(),
      StatusCode::NOT_FOUND,
      "release documentation disabled",
    )?;
  }
  reporter.pass(
    "release-documentation-disabled",
    "Swagger UI, assets and OpenAPI document all return 404",
  );

  let suffix = Uuid::now_v7().simple().to_string();
  let owner_username = format!("bb_owner_{suffix}");
  let owner_password = Uuid::new_v4().simple().to_string();
  let owner_register = register(&client, config, &owner_username, &owner_password)?;
  let owner_org = id(&owner_register["organization_id"], "owner organization")?;
  grant_admin(root, args, &owner_username)?;
  reporter.pass(
    "explicit-admin-provision",
    "new ordinary test account granted admin by the local operator command",
  );
  let owner_token = login(&client, config, &owner_username, &owner_password)?;
  reporter.remember_secret(&owner_password);
  reporter.remember_secret(&owner_token);
  let invalid_login = request_json(
    &client,
    config,
    Method::POST,
    "/auth/login",
    None,
    Some(json!({"username":owner_username,"password":"incorrect-blackbox-password"})),
  )?;
  require_status(
    invalid_login.0,
    StatusCode::UNAUTHORIZED,
    "invalid password rejection",
  )?;
  reporter.pass(
    "registration-and-login",
    "registration, login, and invalid password rejection passed",
  );

  let outsider_username = format!("bb_outsider_{suffix}");
  let outsider_password = Uuid::new_v4().simple().to_string();
  let outsider_register = register(&client, config, &outsider_username, &outsider_password)?;
  let outsider_token = login(&client, config, &outsider_username, &outsider_password)?;
  reporter.remember_secret(&outsider_password);
  reporter.remember_secret(&outsider_token);
  let outsider_org = id(
    &outsider_register["organization_id"],
    "outsider organization",
  )?;
  let (outsider_project_status, outsider_project) = request_json(
    &client,
    config,
    Method::POST,
    "/projects",
    Some(&outsider_token),
    Some(json!({
      "organization_id": outsider_org,
      "name": format!("Blackbox outsider {suffix}"),
      "key": format!("bb-outsider-{suffix}"),
      "visibility": "private",
      "default_branch": "main"
    })),
  )?;
  require_status(
    outsider_project_status,
    StatusCode::OK,
    "outsider private project creation",
  )?;
  let outsider_project_path = data(&outsider_project)["full_path"]
    .as_str()
    .ok_or_else(|| anyhow::anyhow!("outsider project did not return its full path"))?
    .to_owned();

  let project_key = format!("bb-{suffix}");
  let (project_status, project_response) = request_json(
    &client,
    config,
    Method::POST,
    "/projects",
    Some(&owner_token),
    Some(json!({
      "organization_id": owner_org,
      "name": format!("Blackbox {suffix}"),
      "key": project_key,
      "visibility": "private",
      "default_branch": "main"
    })),
  )?;
  require_status(project_status, StatusCode::OK, "private project creation")?;
  let project = data(&project_response);
  let project_id = id(&project["id"], "created project")?;
  let project_path = project["full_path"]
    .as_str()
    .map(str::to_owned)
    .ok_or_else(|| anyhow::anyhow!("created project did not return its full path"))?;

  let private_read = request_json(
    &client,
    config,
    Method::GET,
    &format!("/projects/{project_id}/repository/blob?ref_name=main&path=README.md"),
    Some(&outsider_token),
    None,
  )?;
  require_denied(private_read.0, "private project outsider read")?;
  reporter.pass(
    "private-project-and-outsider-denied",
    format!("created private project {project_id}; outsider cannot read it"),
  );

  let content = format!("blackbox API commit {suffix}\n");
  let (_, commit) = request_json(
    &client,
    config,
    Method::POST,
    &format!("/projects/{project_id}/file-commits"),
    Some(&owner_token),
    Some(json!({
      "branch_name": "main",
      "file_path": "blackbox/api.txt",
      "content": content,
      "message": "Blackbox API commit"
    })),
  )?;
  if data(&commit)["commit_sha"].as_str().is_none() {
    bail!("API file commit did not return a commit SHA");
  }
  let (_, blob) = request_json(
    &client,
    config,
    Method::GET,
    &format!("/projects/{project_id}/repository/blob?ref_name=main&path=blackbox%2Fapi.txt"),
    Some(&owner_token),
    None,
  )?;
  if data(&blob)["content"].as_str() != Some(content.as_str()) {
    bail!("API file read did not match the committed bytes");
  }
  reporter.pass(
    "api-file-commit-read",
    "committed file bytes read back exactly",
  );

  packages::run(
    &client,
    config,
    &project_id,
    &owner_token,
    &outsider_token,
    &suffix,
    reporter,
  )?;
  Ok(Scenario {
    owner_username,
    owner_token,
    outsider_token,
    project_id,
    project_path,
    outsider_project_path,
    suffix,
    git_credential: None,
  })
}

fn register(client: &Client, config: &RunConfig, username: &str, password: &str) -> Result<Value> {
  let (status, response) = request_json(
    client,
    config,
    Method::POST,
    "/auth/register",
    None,
    Some(json!({"username":username,"password":password})),
  )?;
  require_status(status, StatusCode::OK, "account registration")?;
  Ok(data(&response).clone())
}

fn login(client: &Client, config: &RunConfig, username: &str, password: &str) -> Result<String> {
  let (status, response) = request_json(
    client,
    config,
    Method::POST,
    "/auth/login",
    None,
    Some(json!({"username":username,"password":password})),
  )?;
  require_status(status, StatusCode::OK, "account login")?;
  data(&response)["token"]
    .as_str()
    .map(str::to_owned)
    .ok_or_else(|| anyhow::anyhow!("login response did not contain a token"))
}
