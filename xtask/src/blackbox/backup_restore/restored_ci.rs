//! Execute new work through the restored registration, Runner and DIND only.
use crate::blackbox::{Reporter, RunConfig, api};
use anyhow::{Context, Result, bail};
use reqwest::{Method, StatusCode, blocking::Client};
use serde_json::{Value, json};
use std::{
  path::Path,
  time::{Duration, Instant},
};

mod commit;

pub(super) fn run(
  client: &Client,
  config: &RunConfig,
  scenario: &api::Scenario,
  work: &Path,
  reporter: &mut Reporter,
) -> Result<()> {
  let base = format!("/projects/{}", scenario.project_id);
  let branch = "backup-ci-restored";
  let marker = format!("GITY_RESTORED_CI_{}", uuid::Uuid::now_v7().simple());
  post(
    client,
    config,
    scenario,
    &format!("{base}/repository/branches"),
    json!({"name":branch,"ref":"main"}),
  )?;
  let yaml = format!(
    "stages: [test]\njobs:\n  restored-runner:\n    stage: test\n    image: alpine:3.22\n    script:\n      - test -f /etc/alpine-release\n      - test -f .gity-ci.yml\n      - printf '{marker}\\n'\n      - printf '{marker}\\n' > restored-runner-result.txt\n      - test \"$(cat restored-runner-result.txt)\" = '{marker}'\n"
  );
  let sha = commit::push(config, scenario, work, branch, &yaml)?;
  let created = post(
    client,
    config,
    scenario,
    &format!("{base}/pipelines"),
    json!({
      "source":"web","ref_name":branch,"commit_sha":sha,"config_source":"","config_content":""
    }),
  )?;
  let pipeline_id = api::id(&created["pipeline"]["id"], "fresh restored CI pipeline")?;
  let deadline = Instant::now() + Duration::from_secs(300);
  let detail = loop {
    let detail = get(
      client,
      config,
      scenario,
      &format!("{base}/pipelines/{pipeline_id}"),
    )?;
    let pipeline = &detail["pipeline"];
    if pipeline["commit_sha"] != sha
      || pipeline["ref_name"] != branch
      || pipeline["config_content"] != yaml
    {
      bail!("restored CI pipeline did not bind the new branch, SHA and YAML");
    }
    match pipeline["status"].as_str() {
      Some("succeeded") => break detail,
      Some("failed" | "canceled") => bail!("fresh pipeline through restored Runner/DIND failed"),
      _ => {}
    }
    if Instant::now() >= deadline {
      bail!("restored Runner/DIND did not finish fresh pipeline within 300s");
    }
    std::thread::sleep(Duration::from_secs(2));
  };
  let jobs = detail["jobs"]
    .as_array()
    .context("restored pipeline jobs missing")?;
  if jobs.len() != 1
    || jobs[0]["status"] != "succeeded"
    || jobs[0]["pipeline_job"]["image"] != "alpine:3.22"
  {
    bail!("restored CI did not complete the expected Alpine container job");
  }
  let job_id = api::id(&jobs[0]["project_job"]["id"], "restored container job")?;
  let trace = get(
    client,
    config,
    scenario,
    &format!("{base}/jobs/{job_id}/trace"),
  )?;
  let output = trace["output"]
    .as_str()
    .context("restored CI trace missing")?;
  if !output.lines().any(|line| line.trim() == marker) || trace["log"]["exit_code"] != 0 {
    bail!("restored CI trace lacks the new execution marker or zero exit code");
  }
  let (status, _) = api::request_json(
    client,
    config,
    Method::GET,
    &format!("{base}/jobs/{job_id}/trace"),
    Some(&scenario.outsider_token),
    None,
  )?;
  api::require_denied(status, "restored CI trace outsider access")?;
  reporter.pass("offline-restore-fresh-container-ci", format!("restored Runner/DIND executed SSH-pushed new {branch} commit {sha}; pipeline {pipeline_id}, container job {job_id}, exact trace marker and exit 0; outsider denied"));
  Ok(())
}

fn post(
  client: &Client,
  config: &RunConfig,
  scenario: &api::Scenario,
  path: &str,
  body: Value,
) -> Result<Value> {
  let (status, value) = api::request_json(
    client,
    config,
    Method::POST,
    path,
    Some(&scenario.owner_token),
    Some(body),
  )?;
  api::require_status(status, StatusCode::OK, "submit fresh restored CI operation")?;
  Ok(api::data(&value).clone())
}

fn get(client: &Client, config: &RunConfig, scenario: &api::Scenario, path: &str) -> Result<Value> {
  let (status, value) = api::request_json(
    client,
    config,
    Method::GET,
    path,
    Some(&scenario.owner_token),
    None,
  )?;
  api::require_status(status, StatusCode::OK, "read fresh restored CI result")?;
  Ok(api::data(&value).clone())
}
