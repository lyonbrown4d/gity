use crate::api;
use serde_json::{Value, json};
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Status {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}
impl Status {
    pub fn parse(value: &str) -> Self {
        match value {
            "running" => Self::Running,
            "succeeded" => Self::Succeeded,
            "failed" => Self::Failed,
            "cancelled" | "canceled" => Self::Cancelled,
            _ => Self::Pending,
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
    pub fn terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Pending => "Queued and waiting for an available runner.",
            Self::Running => "Claimed by a runner and currently executing.",
            Self::Succeeded => "Completed successfully with no reported errors.",
            Self::Failed => "Finished with an error. Inspect the failure output below.",
            Self::Cancelled => "Stopped before completion.",
        }
    }
    pub fn guidance(self) -> &'static str {
        match self {
            Self::Failed => {
                "For failures, compare last_error with the pipeline trace and inspect artifacts for test reports, screenshots, or partial build output."
            }
            Self::Pending => {
                "Trace and artifacts are not expected until a runner claims the job and starts the script."
            }
            Self::Running => {
                "Trace output may still be streaming. If artifacts are missing, wait until the runner completes upload."
            }
            Self::Succeeded => {
                "A successful job can still have empty output. Use artifacts as the durable evidence when the script uploaded files."
            }
            Self::Cancelled => {
                "Cancelled jobs may have partial trace output but should not be expected to produce complete artifacts."
            }
        }
    }
    pub fn empty_output(self) -> &'static str {
        match self {
            Self::Pending => {
                "No log is available because the job is still queued and has not been claimed by a runner."
            }
            Self::Running => {
                "The runner has not stored output yet. Reload after the next trace flush or after completion."
            }
            Self::Failed => {
                "The job failed without stored result output. Use last_error, attempt count, worker assignment, and the pipeline trace to diagnose it."
            }
            Self::Succeeded => {
                "The job succeeded without a stored result payload. This can be normal for jobs that only update status or upload artifacts."
            }
            Self::Cancelled => {
                "The job was cancelled before a durable result payload was recorded."
            }
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Stats {
    pub pending: usize,
    pub running: usize,
    pub succeeded: usize,
    pub failed: usize,
}
impl Stats {
    pub fn from_rows(value: &Value) -> Self {
        let mut result = Self::default();
        for row in api::list(value) {
            match Status::parse(&api::text(&row, "status")) {
                Status::Pending => result.pending += 1,
                Status::Running => result.running += 1,
                Status::Succeeded => result.succeeded += 1,
                Status::Failed => result.failed += 1,
                Status::Cancelled => {}
            }
        }
        result
    }
    pub fn count(&self, status: Status) -> usize {
        match status {
            Status::Pending => self.pending,
            Status::Running => self.running,
            Status::Succeeded => self.succeeded,
            Status::Failed => self.failed,
            Status::Cancelled => 0,
        }
    }
    pub fn diagnostic(&self) -> &'static str {
        if self.failed > 0 {
            "Start with failed jobs: read last_error, open the related pipeline job trace when available, and verify whether artifacts were expected but missing."
        } else if self.pending > 0 && self.running == 0 {
            "Queued jobs with no active workers usually indicate runner capacity, heartbeat, or tag-matching problems."
        } else if self.running > 0 {
            "Running jobs may not have flushed output yet. Reload after completion or inspect the pipeline job trace for live diagnostics."
        } else {
            "Use this queue as the low-level CI control plane: result text, last_error, attempts, worker assignment, trace, and artifacts together explain job outcomes."
        }
    }
}
pub(super) fn enqueue(kind: &str, payload: &str, attempts: &str) -> Result<Value, &'static str> {
    let attempts = attempts
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or("Max attempts must be a positive number")?;
    Ok(
        json!({"kind":if kind.trim().is_empty(){"noop"}else{kind.trim()},"payload":payload.trim(),"max_attempts":attempts}),
    )
}
#[cfg(test)]
#[path = "../../tests/jobs.rs"]
mod tests;
