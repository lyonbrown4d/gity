use super::{BlackboxArgs, Mode, Reporter, RunConfig, api, runtime};
use anyhow::{Context, Result, bail};
use reqwest::{Method, StatusCode, blocking::Client};
use serde_json::{Value, json};
use std::{
  path::Path,
  time::{Duration, Instant},
};

pub fn run(
  root: &Path,
  args: &BlackboxArgs,
  config: &RunConfig,
  scenario: &api::Scenario,
  reporter: &mut Reporter,
) -> Result<()> {
  let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
  let yaml = format!(
    "stages: [test]\njobs:\n  blackbox-runner:\n    stage: test\n    image: alpine:3.22\n    script:\n      - test -f .gity-ci.yml\n      - printf '{0}\\n' > runner-result.txt\n      - test \"$(cat runner-result.txt)\" = '{0}'\n",
    scenario.suffix
  );
  let (_, committed) = api::request_json(
    &client,
    config,
    Method::POST,
    &format!("/projects/{}/file-commits", scenario.project_id),
    Some(&scenario.owner_token),
    Some(json!({
      "branch_name":"main",
      "file_path":".gity-ci.yml",
      "content":yaml,
      "message":"Blackbox runner fixture"
    })),
  )?;
  let commit_sha = api::data(&committed)["commit_sha"]
    .as_str()
    .ok_or_else(|| anyhow::anyhow!("CI fixture commit did not return a commit SHA"))?;
  let runner_name = format!("blackbox-{}", scenario.suffix);
  let (_, registration) = api::request_json(
    &client,
    config,
    Method::POST,
    &format!("/projects/{}/runners", scenario.project_id),
    Some(&scenario.owner_token),
    Some(json!({"name":runner_name,"description":"task-owned shell runner","tags":""})),
  )?;
  let token = api::data(&registration)["token"]
    .as_str()
    .ok_or_else(|| anyhow::anyhow!("runner registration did not return its one-time token"))?
    .to_owned();
  reporter.remember_secret(&token);
  if args.mode == Mode::Distributed {
    runtime::start_distributed_runner(root, args, &token)?;
  }
  let (_, pipeline) = api::request_json(
    &client,
    config,
    Method::POST,
    &format!("/projects/{}/pipelines", scenario.project_id),
    Some(&scenario.owner_token),
    Some(json!({
      "source":"web",
      "ref_name":"main",
      "commit_sha":commit_sha,
      "config_source":"",
      "config_content":""
    })),
  )?;
  let pipeline_id = api::id(&api::data(&pipeline)["pipeline"]["id"], "CI pipeline")?;
  let deadline = Instant::now() + Duration::from_secs(180);
  loop {
    let detail = client
      .get(format!(
        "{}/projects/{}/pipelines/{pipeline_id}",
        config.api_url, scenario.project_id
      ))
      .bearer_auth(&scenario.owner_token)
      .send()
      .context("poll CI pipeline state")?;
    if detail.status() != StatusCode::OK {
      bail!("CI pipeline polling failed with HTTP {}", detail.status());
    }
    let body: Value = detail.json().context("decode CI pipeline state")?;
    let pipeline = api::data(&body)["pipeline"].clone();
    match pipeline["status"].as_str().unwrap_or_default() {
      "succeeded" => {
        reporter.pass(
          "ci-runner-execution",
          format!("registered Runner completed pipeline {pipeline_id} at {commit_sha}"),
        );
        return Ok(());
      }
      "failed" | "canceled" => {
        let trace = failure_trace(&client, config, scenario, api::data(&body));
        bail!(
          "blackbox CI pipeline ended as {}: {trace}",
          pipeline["status"]
        );
      }
      _ => {}
    }
    if Instant::now() >= deadline {
      bail!("CI pipeline did not finish within 180 seconds");
    }
    std::thread::sleep(Duration::from_secs(2));
  }
}

fn failure_trace(
  client: &Client,
  config: &RunConfig,
  scenario: &api::Scenario,
  detail: &Value,
) -> String {
  let Some(jobs) = detail["jobs"].as_array() else {
    return "no job details returned".to_owned();
  };
  jobs
    .iter()
    .filter_map(|job| {
      let id = job["project_job"]["id"].as_str()?;
      let response = client
        .get(format!(
          "{}/projects/{}/jobs/{id}/trace",
          config.api_url, scenario.project_id
        ))
        .bearer_auth(&scenario.owner_token)
        .send()
        .ok()?
        .error_for_status()
        .ok()?;
      let body: Value = response.json().ok()?;
      let output = api::data(&body)["output"]
        .as_str()?
        .chars()
        .take(4096)
        .collect::<String>();
      Some(format!("job {id}: {output}"))
    })
    .collect::<Vec<_>>()
    .join("\n")
}
