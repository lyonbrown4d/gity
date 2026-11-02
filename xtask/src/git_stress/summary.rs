use super::{Config, workload::Stats};
use anyhow::{Context, Result};
use serde::Serialize;
use std::{collections::BTreeMap, fs, path::Path, time::Duration};

#[derive(Debug, Serialize)]
pub(super) struct Metrics {
  count: usize,
  fail: usize,
  avg_ms: f64,
  p50_ms: f64,
  p95_ms: f64,
  p99_ms: f64,
  max_ms: f64,
}
#[derive(Serialize)]
pub(super) struct Summary {
  schema_version: u32,
  run_id: String,
  transport: super::Transport,
  mode: super::Mode,
  workers: usize,
  clone_depth: usize,
  seed_applied: bool,
  seed_files: usize,
  bytes_per_file: Option<u64>,
  seed_commits: usize,
  latency_scope: &'static str,
  wait_strategy: &'static str,
  setup: Phase,
  measurement: Phase,
}
#[derive(Serialize)]
struct Phase {
  #[serde(skip_serializing_if = "Option::is_none")]
  started_unix_seconds: Option<f64>,
  #[serde(skip_serializing_if = "Option::is_none")]
  finished_unix_seconds: Option<f64>,
  elapsed_seconds: f64,
  requested_duration_seconds: Option<u64>,
  count: usize,
  fail: usize,
  throughput_ops_per_second: f64,
  successful_ops_per_second: f64,
  operations: BTreeMap<String, Metrics>,
}
impl Summary {
  pub fn new(
    config: &Config,
    run_id: &str,
    stats: &Stats,
    setup_elapsed: Duration,
    elapsed: Duration,
    window_unix_seconds: (f64, f64),
  ) -> Self {
    let stats = stats.lock().unwrap();
    let setup = stats
      .iter()
      .filter(|(name, _)| name.starts_with("setup."))
      .map(|(name, v)| (name.clone(), metrics(v)))
      .collect();
    let measurement = stats
      .iter()
      .filter(|(name, _)| !name.starts_with("setup."))
      .map(|(name, v)| (name.clone(), metrics(v)))
      .collect();
    let mut measurement = Phase::new(measurement, elapsed, config.duration_seconds);
    measurement.started_unix_seconds = Some(window_unix_seconds.0);
    measurement.finished_unix_seconds = Some(window_unix_seconds.1);
    Self {
      schema_version: 1,
      run_id: run_id.into(),
      transport: config.transport,
      mode: config.mode,
      workers: config.workers,
      clone_depth: config.clone_depth,
      seed_applied: config.project_url.is_empty(),
      seed_files: config.seed_files,
      bytes_per_file: config.bytes_per_file,
      seed_commits: config.seed_commits,
      latency_scope: "end-to-end Git client operation; push includes local checkout/add/commit",
      wait_strategy: "wait-timeout native child completion, process-group cancellation",
      setup: Phase::new(setup, setup_elapsed, None),
      measurement,
    }
  }
  pub fn write(&self, root: &Path, path: &Path) -> Result<()> {
    let path = if path.is_absolute() {
      path.to_owned()
    } else {
      root.join(path)
    };
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(self)?).context("write Git stress JSON summary")
  }
  pub fn print(&self) {
    println!("\nGit client stress summary (client end-to-end latency)");
    for (name, m) in self
      .setup
      .operations
      .iter()
      .chain(&self.measurement.operations)
    {
      println!(
        "{name:14} count={} fail={} avg={:.2}ms p50={:.2}ms p95={:.2}ms p99={:.2}ms max={:.2}ms",
        m.count, m.fail, m.avg_ms, m.p50_ms, m.p95_ms, m.p99_ms, m.max_ms
      );
    }
    println!(
      "Measured {:.3}s: count={} fail={} throughput={:.2}ops/s successful={:.2}ops/s; setup {:.3}s",
      self.measurement.elapsed_seconds,
      self.measurement.count,
      self.measurement.fail,
      self.measurement.throughput_ops_per_second,
      self.measurement.successful_ops_per_second,
      self.setup.elapsed_seconds
    );
  }
}
impl Phase {
  fn new(
    operations: BTreeMap<String, Metrics>,
    elapsed: Duration,
    requested_duration_seconds: Option<u64>,
  ) -> Self {
    let count = operations.values().map(|m| m.count).sum();
    let fail = operations.values().map(|m| m.fail).sum();
    let seconds = elapsed.as_secs_f64();
    Self {
      started_unix_seconds: None,
      finished_unix_seconds: None,
      elapsed_seconds: seconds,
      requested_duration_seconds,
      count,
      fail,
      throughput_ops_per_second: if seconds > 0.0 {
        count as f64 / seconds
      } else {
        0.0
      },
      successful_ops_per_second: if seconds > 0.0 {
        (count - fail) as f64 / seconds
      } else {
        0.0
      },
      operations,
    }
  }
}
fn metrics(samples: &[(Duration, bool)]) -> Metrics {
  let mut values = samples
    .iter()
    .map(|(d, _)| d.as_secs_f64() * 1000.0)
    .collect::<Vec<_>>();
  values.sort_by(f64::total_cmp);
  Metrics {
    count: values.len(),
    fail: samples.iter().filter(|(_, ok)| !*ok).count(),
    avg_ms: values.iter().sum::<f64>() / values.len().max(1) as f64,
    p50_ms: percentile(&values, 0.5),
    p95_ms: percentile(&values, 0.95),
    p99_ms: percentile(&values, 0.99),
    max_ms: values.last().copied().unwrap_or_default(),
  }
}
fn percentile(values: &[f64], quantile: f64) -> f64 {
  let index = (values.len() as f64 * quantile).ceil() as usize;
  values
    .get(index.saturating_sub(1))
    .copied()
    .unwrap_or_default()
}
#[cfg(test)]
#[path = "summary_tests.rs"]
mod tests;
