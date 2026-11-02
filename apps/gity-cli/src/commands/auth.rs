//! `gity auth` — sign in, inspect, and clear stored credentials.
use crate::client::{Client, Options};
use crate::config;
use crate::format;
use anyhow::{Result, bail};
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand)]
pub enum AuthCommand {
  /// Store credentials. `--token` keeps an existing token; username/password
  /// signs in and mints a fresh personal access token for the CLI.
  Login {
    #[arg(long)]
    host: Option<String>,
    #[arg(long, env = "GITY_TOKEN", hide_env_values = true)]
    token: Option<String>,
    #[arg(long)]
    username: Option<String>,
    #[arg(long)]
    password: Option<String>,
    /// Name recorded for the minted personal access token
    #[arg(long, default_value = "gity-cli")]
    name: String,
  },
  /// Show the active server and account
  Status,
  /// Remove the stored profile
  Logout,
}

pub async fn run(command: AuthCommand, options: &Options) -> Result<()> {
  match command {
    AuthCommand::Login {
      host,
      token,
      username,
      password,
      name,
    } => login(options, host, token, username, password, name).await,
    AuthCommand::Status => status(options).await,
    AuthCommand::Logout => logout(options),
  }
}

async fn login(
  options: &Options,
  host: Option<String>,
  token: Option<String>,
  username: Option<String>,
  password: Option<String>,
  name: String,
) -> Result<()> {
  let host = host.or_else(|| options.host.clone()).ok_or_else(|| {
    anyhow::anyhow!("--host is required, for example --host http://127.0.0.1:8080")
  })?;
  let host = host.trim_end_matches('/').to_owned();
  let token = match (token, username, password) {
    (Some(token), _, _) => token,
    (None, Some(username), Some(password)) => {
      mint_token(&host, &username, &password, &name).await?
    }
    (None, Some(_), None) => bail!("--password is required together with --username"),
    _ => bail!("pass --token, or --username and --password"),
  };
  let client = Client::new(&host, &token);
  let me = client.get("/users/me").await?;
  config::store_host(options, &host, &token, me["username"].as_str())?;
  println!(
    "{} {} on {}",
    format::paint("signed in as", "32"),
    format::text(&me, "username"),
    host
  );
  println!("credentials stored in {}", options.config_file.display());
  Ok(())
}

/// Password login yields a short-lived session; the CLI stores a personal token.
async fn mint_token(host: &str, username: &str, password: &str, name: &str) -> Result<String> {
  let http = reqwest::Client::new();
  let base = format!("{}/api/v1", host.trim_end_matches('/'));
  let session = post(
    &http,
    &format!("{base}/auth/login"),
    None,
    json!({
      "username": username,
      "password": password,
    }),
  )
  .await?;
  let user_id = session["user_id"]
    .as_str()
    .ok_or_else(|| anyhow::anyhow!("login response did not include a user id"))?
    .to_owned();
  let token = session["token"]
    .as_str()
    .ok_or_else(|| anyhow::anyhow!("login response did not include a token"))?
    .to_owned();
  let created = post(
    &http,
    &format!("{base}/users/{user_id}/tokens"),
    Some(&token),
    json!({ "name": name }),
  )
  .await?;
  created["token"]
    .as_str()
    .map(str::to_owned)
    .ok_or_else(|| anyhow::anyhow!("token creation response did not include a token"))
}

async fn post(
  http: &reqwest::Client,
  url: &str,
  token: Option<&str>,
  body: serde_json::Value,
) -> Result<serde_json::Value> {
  let mut request = http.post(url).json(&body);
  if let Some(token) = token {
    request = request.bearer_auth(token);
  }
  let response = request.send().await?;
  let status = response.status();
  let payload: serde_json::Value = response.json().await?;
  if !status.is_success() || payload["code"].as_i64() != Some(0) {
    bail!(
      "{} (HTTP {})",
      payload["message"].as_str().unwrap_or("request failed"),
      status.as_u16()
    );
  }
  Ok(payload["data"].clone())
}

async fn status(options: &Options) -> Result<()> {
  let resolved = config::resolve(options)?;
  let host = resolved.host;
  let client = Client::from_options(options)?;
  let me = client.get("/users/me").await?;
  if options.json {
    format::print_json(&json!({ "host": host, "user": me }));
    return Ok(());
  }
  println!("host:  {host}");
  println!("user:  {}", format::text(&me, "username"));
  println!("admin: {}", format::field(&me, "is_super_admin"));
  Ok(())
}

fn logout(options: &Options) -> Result<()> {
  let Some(host) = config::load(&options.config_file).host else {
    println!("no stored credentials");
    return Ok(());
  };
  config::remove_host(options, &host)?;
  println!("signed out of {host}");
  Ok(())
}
