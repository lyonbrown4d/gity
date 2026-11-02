//! Real-image collaboration and durable merge recovery acceptance.
use super::{BlackboxArgs, Mode, Reporter, RunConfig, api, process, runtime};
use anyhow::{Context, Result, ensure};
use reqwest::{Method, StatusCode, blocking::Client};
use serde_json::{Value, json};
use std::{path::Path, time::Duration};

mod database;
mod fault;
mod flow;
mod native;
mod normal;

struct Fixture<'a> {
  client: Client,
  config: &'a RunConfig,
  owner: &'a str,
  outsider: &'a str,
  project: String,
  project_path: String,
  reviewer: String,
  reviewer_id: i64,
  milestone: String,
}
impl Fixture<'_> {
  fn request(
    &self,
    method: Method,
    suffix: &str,
    token: &str,
    body: Option<Value>,
  ) -> Result<(StatusCode, Value)> {
    api::request_json(
      &self.client,
      self.config,
      method,
      &format!("/projects/{}{}", self.project, suffix),
      Some(token),
      body,
    )
  }
  fn ok(&self, method: Method, suffix: &str, token: &str, body: Option<Value>) -> Result<Value> {
    let (status, value) = self.request(method, suffix, token, body)?;
    api::require_status(status, StatusCode::OK, suffix)?;
    Ok(api::data(&value).clone())
  }
  fn get(&self, suffix: &str) -> Result<Value> {
    self.ok(Method::GET, suffix, self.owner, None)
  }
  fn commit(&self, branch: &str, path: &str) -> Result<String> {
    let value = self.ok(
      Method::POST,
      "/file-commits",
      self.owner,
      Some(json!({
        "branch_name":branch,"file_path":path,"content":path,"message":format!("Blackbox {path}")
      })),
    )?;
    text(&value, "commit_sha")
  }
  fn main_sha(&self) -> Result<String> {
    let branches = self.get("/repository/branches")?;
    let main = branches
      .as_array()
      .context("branches response must be an array")?
      .iter()
      .find(|branch| branch["name"] == "main")
      .context("main branch missing")?;
    main
      .get("last_commit_sha")
      .or_else(|| main.get("hash"))
      .and_then(Value::as_str)
      .map(str::to_owned)
      .context("branch commit SHA missing")
  }
  fn create_merge(&self, branch: &str) -> Result<Value> {
    self.ok(
      Method::POST,
      "/repository/branches",
      self.owner,
      Some(json!({"name":branch,"source_ref":"main"})),
    )?;
    self.commit(branch, &format!("blackbox/{branch}.txt"))?;
    self.ok(Method::POST, "/merge-requests", self.owner, Some(json!({
      "title":format!("Blackbox {branch}"),"source_branch":branch,"target_branch":"main","milestone_id":self.milestone
    })))
  }
  fn approve(&self, iid: &str) -> Result<()> {
    self.ok(
      Method::POST,
      &format!("/merge-requests/{iid}/approve"),
      &self.reviewer,
      Some(json!({})),
    )?;
    Ok(())
  }
}

fn text(value: &Value, key: &str) -> Result<String> {
  value[key]
    .as_str()
    .map(str::to_owned)
    .with_context(|| format!("missing string {key}"))
}

pub(super) fn run(
  root: &Path,
  args: &BlackboxArgs,
  config: &RunConfig,
  scenario: &api::Scenario,
  work: &Path,
  reporter: &mut Reporter,
) -> Result<()> {
  let fixture = flow::fixture(config, scenario, reporter)?;
  normal::run(&fixture, reporter)?;
  fault::run(root, args, &fixture, work, reporter)
}
