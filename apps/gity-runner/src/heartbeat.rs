//! Lease keep-alive shared by script and index jobs.
//!
//! The heartbeat both renews the lease and detects cancellation: when the server
//! no longer lists this attempt as running, the job has been canceled or the
//! lease was lost, so execution must stop.
use crate::client::Client;
use anyhow::{Context, Result};
use serde_json::json;
use std::time::Duration;

pub(crate) fn spawn(
  client: &Client,
  id: String,
  attempt: i64,
) -> tokio::task::JoinHandle<Result<()>> {
  let client = client.clone();
  tokio::spawn(async move {
    let mut heartbeat = tokio::time::interval(Duration::from_secs(15));
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
      heartbeat.tick().await;
      let response = tokio::time::timeout(
        Duration::from_secs(15),
        client.post("/runners/heartbeat", json!({})),
      )
      .await
      .context("runner heartbeat timed out")??;
      if response["running_job_attempts"][&id].as_i64() != Some(attempt) {
        return Err(anyhow::anyhow!("CI job was canceled or its lease was lost"));
      }
    }
  })
}
