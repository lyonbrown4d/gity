use crate::blackbox::{RunConfig, api, git};
use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::{Method, StatusCode, blocking::Client};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;

pub(super) struct Fixture {
  issue_path: String,
  issue: Value,
  merge_path: String,
  merge_request: Value,
  attachment: String,
  lfs: String,
  bytes: Vec<u8>,
  refs: Vec<u8>,
}

pub(super) fn seed(
  client: &Client,
  config: &RunConfig,
  scenario: &api::Scenario,
  work: &Path,
) -> Result<Fixture> {
  let base = format!("/projects/{}", scenario.project_id);
  let call = |path: &str, body: Value| -> Result<Value> {
    let (status, value) = api::request_json(
      client,
      config,
      Method::POST,
      path,
      Some(&scenario.owner_token),
      Some(body),
    )?;
    api::require_status(status, StatusCode::OK, "seed backup record")?;
    Ok(api::data(&value).clone())
  };
  let issue = call(
    &format!("{base}/issues"),
    json!({"title":"Offline backup issue","description":"exact persistent collaboration record"}),
  )?;
  let iid = api::id(&issue["number"], "backup issue IID")?;
  let issue_path = format!("{base}/issues/{iid}");
  let bytes: Vec<u8> = (0..65536).map(|index| (index % 251) as u8).collect();
  let attachment = call(
    &format!("{issue_path}/attachments"),
    json!({"file_name":"backup.bin","content_base64":STANDARD.encode(&bytes),"content_type":"application/octet-stream"}),
  )?;
  let attachment = format!(
    "{issue_path}/attachments/{}/raw",
    api::id(&attachment["id"], "backup attachment")?
  );
  let oid = hex::encode(Sha256::digest(&bytes));
  let lfs = format!(
    "{}/{}.git/info/lfs/objects/{oid}",
    config.http_url, scenario.project_path
  );
  api::require_status(
    client
      .put(&lfs)
      .bearer_auth(&scenario.owner_token)
      .body(bytes.clone())
      .send()?
      .status(),
    StatusCode::OK,
    "seed LFS backup bytes",
  )?;
  call(
    &format!("{base}/repository/branches"),
    json!({"name":"backup-roundtrip","ref":"main"}),
  )?;
  call(
    &format!("{base}/file-commits"),
    json!({"branch_name":"backup-roundtrip","file_path":"backup-roundtrip.txt","content":"persistent MR branch\n","message":"offline backup fixture"}),
  )?;
  let merge_request = call(
    &format!("{base}/merge-requests"),
    json!({"title":"Offline backup MR","source_branch":"backup-roundtrip","target_branch":"main"}),
  )?;
  let merge_path = format!(
    "{base}/merge-requests/{}",
    api::id(&merge_request["iid"], "backup MR IID")?
  );
  let issue = get_record(client, config, scenario, &issue_path)?;
  let merge_request = get_record(client, config, scenario, &merge_path)?;
  let credential = scenario
    .git_credential
    .as_ref()
    .context("backup needs pinned SSH credentials")?;
  let url = format!(
    "ssh://git@{}:{}/{}.git",
    config.ssh_host, config.ssh_port, scenario.project_path
  );
  settle_jobs(client, config, scenario)?;
  let refs = git_refs(work, credential, &url)?;
  Ok(Fixture {
    issue_path,
    issue,
    merge_path,
    merge_request,
    attachment,
    lfs,
    bytes,
    refs,
  })
}

pub(super) fn verify(
  client: &Client,
  config: &RunConfig,
  scenario: &api::Scenario,
  work: &Path,
  fixture: &Fixture,
  phase: &str,
) -> Result<()> {
  for (path, expected) in [
    (&fixture.issue_path, &fixture.issue),
    (&fixture.merge_path, &fixture.merge_request),
  ] {
    if get_record(client, config, scenario, path)? != *expected {
      bail!("restored collaboration record changed");
    }
    let (status, _) = api::request_json(
      client,
      config,
      Method::GET,
      path,
      Some(&scenario.outsider_token),
      None,
    )?;
    api::require_denied(status, "restored collaboration outsider access")?;
  }
  for url in [
    format!("{}{}", config.api_url, fixture.attachment),
    fixture.lfs.clone(),
  ] {
    let response = client.get(&url).bearer_auth(&scenario.owner_token).send()?;
    api::require_status(
      response.status(),
      StatusCode::OK,
      "restored binary object read",
    )?;
    if response.bytes()?.as_ref() != fixture.bytes {
      bail!("restored binary object bytes differ");
    }
    api::require_denied(
      client
        .get(&url)
        .bearer_auth(&scenario.outsider_token)
        .send()?
        .status(),
      "restored object outsider access",
    )?;
  }
  let credential = scenario
    .git_credential
    .as_ref()
    .context("backup needs pinned SSH credentials")?;
  let url = format!(
    "ssh://git@{}:{}/{}.git",
    config.ssh_host, config.ssh_port, scenario.project_path
  );
  if git_refs(work, credential, &url)? != fixture.refs {
    bail!("restored Git refs differ from offline cut");
  }
  let clone = work.join(format!("git-{phase}"));
  git::commands::run_git(
    work,
    &credential.private_key,
    &credential.known_hosts,
    [
      "clone",
      "--branch",
      "backup-roundtrip",
      &url,
      clone.to_str().context("clone path must be UTF-8")?,
    ],
    "strict-pinned SSH clone after restore",
  )?;
  if std::fs::read(clone.join("backup-roundtrip.txt"))? != b"persistent MR branch\n" {
    bail!("restored clone bytes differ");
  }
  if std::fs::read(clone.join("ssh-blackbox.txt"))? != b"pushed by the first clone\n" {
    bail!("restored original Git commit bytes differ");
  }
  let unknown = work.join("ssh-unknown-ed25519");
  if git::commands::output(work, &unknown, &credential.known_hosts, ["ls-remote", &url]).is_ok() {
    bail!("restored SSH accepted an unregistered key");
  }
  Ok(())
}

fn get_record(
  client: &Client,
  config: &RunConfig,
  scenario: &api::Scenario,
  path: &str,
) -> Result<Value> {
  let (status, value) = api::request_json(
    client,
    config,
    Method::GET,
    path,
    Some(&scenario.owner_token),
    None,
  )?;
  api::require_status(status, StatusCode::OK, "read backup collaboration record")?;
  Ok(api::data(&value).clone())
}

fn git_refs(work: &Path, credential: &git::GitCredential, url: &str) -> Result<Vec<u8>> {
  // Existing helper configures strict checking with the original pinned known_hosts.
  let output = git::commands::output(
    work,
    &credential.private_key,
    &credential.known_hosts,
    ["ls-remote", url],
  )?;
  Ok(output)
}

fn settle_jobs(client: &Client, config: &RunConfig, scenario: &api::Scenario) -> Result<()> {
  let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
  loop {
    let jobs = get_record(
      client,
      config,
      scenario,
      &format!("/projects/{}/jobs", scenario.project_id),
    )?;
    let jobs = jobs
      .as_array()
      .context("job list did not return an array")?;
    if jobs.iter().all(|job| {
      matches!(
        job["status"].as_str(),
        Some("succeeded" | "failed" | "canceled" | "skipped")
      )
    }) {
      return Ok(());
    }
    if std::time::Instant::now() >= deadline {
      bail!("project jobs did not settle before offline backup");
    }
    std::thread::sleep(std::time::Duration::from_secs(2));
  }
}
