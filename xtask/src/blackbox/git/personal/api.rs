use crate::blackbox::{Reporter, RunConfig, api};
use anyhow::{Context, Result, bail};
use reqwest::{Method, StatusCode, blocking::Client};
use serde_json::{Value, json};
use std::{path::Path, process::Command};

pub(super) struct Account {
  pub token: String,
  pub id: String,
  username: String,
  password: String,
}
pub(super) fn register(
  client: &Client,
  config: &RunConfig,
  suffix: &str,
  reporter: &mut Reporter,
) -> Result<Account> {
  let username = format!("bb_ssh_{suffix}");
  let password = uuid::Uuid::new_v4().simple().to_string();
  reporter.remember_secret(&password);
  let value = ok(
    client,
    config,
    Method::POST,
    "/auth/register",
    None,
    Some(json!({"username":username,"password":password})),
    "register ordinary SSH account",
  )?;
  let token = value["token"]
    .as_str()
    .context("SSH registration token missing")?
    .to_owned();
  reporter.remember_secret(&token);
  let profile = ok(
    client,
    config,
    Method::GET,
    "/users/me",
    Some(&token),
    None,
    "read ordinary SSH account",
  )?;
  if profile["is_super_admin"] != false {
    bail!("personal SSH test account unexpectedly has administrator privileges");
  }
  Ok(Account {
    token,
    id: api::id(&profile["id"], "ordinary SSH account")?,
    username,
    password,
  })
}
pub(super) fn ok(
  client: &Client,
  config: &RunConfig,
  method: Method,
  path: &str,
  token: Option<&str>,
  body: Option<Value>,
  action: &str,
) -> Result<Value> {
  let (status, value) = api::request_json(client, config, method, path, token, body)?;
  api::require_status(status, StatusCode::OK, action)?;
  Ok(api::data(&value).clone())
}
pub(super) fn role(
  client: &Client,
  config: &RunConfig,
  scenario: &api::Scenario,
  account: &Account,
  role: Option<&str>,
) -> Result<()> {
  let path = format!("/projects/{}/members/{}", scenario.project_id, account.id);
  let (method, body) = role.map_or((Method::DELETE, None), |role| {
    (Method::PATCH, Some(json!({"role":role})))
  });
  ok(
    client,
    config,
    method,
    &path,
    Some(&scenario.owner_token),
    body,
    "change personal SSH membership",
  )?;
  Ok(())
}
#[expect(
  clippy::too_many_arguments,
  reason = "Black-box inputs keep transport, key material and expected outcomes explicit"
)]
pub(super) fn negative_inputs(
  client: &Client,
  config: &RunConfig,
  scenario: &api::Scenario,
  account: &Account,
  key: &Path,
  public: &str,
  deploy_key: &Path,
  reporter: &mut Reporter,
) -> Result<()> {
  let private = std::fs::read_to_string(key)?;
  reporter.remember_secret(&private);
  expect(
    client,
    config,
    Method::POST,
    "/users/me/ssh-keys",
    &account.token,
    json!({"title":"invalid-private","public_key":private}),
    StatusCode::BAD_REQUEST,
    "reject private SSH key",
  )?;
  expect(
    client,
    config,
    Method::POST,
    "/users/me/ssh-keys",
    &account.token,
    json!({"title":"duplicate","public_key":public}),
    StatusCode::CONFLICT,
    "reject duplicate personal SSH fingerprint",
  )?;
  expect(
    client,
    config,
    Method::POST,
    &format!("/projects/{}/deploy-keys", scenario.project_id),
    &scenario.owner_token,
    json!({"title":"cross-mode","public_key":public,"can_push":true}),
    StatusCode::CONFLICT,
    "reject personal fingerprint as deploy key",
  )?;
  let deploy_public = crate::blackbox::process::run_output(
    Command::new("ssh-keygen")
      .args(["-y", "-f"])
      .arg(deploy_key),
    "derive existing deploy public key",
  )?;
  let deploy_public = String::from_utf8(deploy_public.stdout).context("deploy public key UTF-8")?;
  expect(
    client,
    config,
    Method::POST,
    "/users/me/ssh-keys",
    &account.token,
    json!({"title":"cross-mode","public_key":deploy_public}),
    StatusCode::CONFLICT,
    "reject deploy fingerprint as personal key",
  )?;
  Ok(())
}
#[expect(
  clippy::too_many_arguments,
  reason = "Black-box requests name both the expected status and the operation"
)]
pub(super) fn expect(
  client: &Client,
  config: &RunConfig,
  method: Method,
  path: &str,
  token: &str,
  body: Value,
  expected: StatusCode,
  action: &str,
) -> Result<()> {
  let (status, _) = api::request_json(client, config, method, path, Some(token), Some(body))?;
  api::require_status(status, expected, action)
}

pub(super) fn status(
  client: &Client,
  config: &RunConfig,
  scenario: &api::Scenario,
  account: &Account,
  status: &str,
) -> Result<()> {
  ok(
    client,
    config,
    Method::PATCH,
    &format!("/users/{}", account.id),
    Some(&scenario.owner_token),
    Some(json!({"status":status})),
    "change SSH key creator status",
  )?;
  Ok(())
}
pub(super) fn login(
  client: &Client,
  config: &RunConfig,
  account: &mut Account,
  reporter: &mut Reporter,
) -> Result<()> {
  let value = ok(
    client,
    config,
    Method::POST,
    "/auth/login",
    None,
    Some(json!({"username":account.username,"password":account.password})),
    "renew reactivated SSH account session",
  )?;
  account.token = value["token"]
    .as_str()
    .context("reactivated SSH account token missing")?
    .into();
  reporter.remember_secret(&account.token);
  Ok(())
}
