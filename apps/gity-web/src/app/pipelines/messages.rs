use super::model::{Job, Pipeline, State};

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Summary {
    pub total: usize,
    pub pending: usize,
    pub running: usize,
    pub failed: usize,
    pub blocked: usize,
    pub artifacts: usize,
}
impl Summary {
    pub fn new(jobs: &[Job]) -> Self {
        let mut result = Self {
            total: jobs.len(),
            ..Self::default()
        };
        for job in jobs {
            match job.state {
                State::Pending => result.pending += 1,
                State::Running => result.running += 1,
                State::Failed => result.failed += 1,
                State::Blocked => result.blocked += 1,
                _ => {}
            }
            result.artifacts += usize::from(!job.artifacts.is_empty());
        }
        result
    }
    pub fn guidance(&self, pipeline: &Pipeline, can_mutate: bool) -> &'static str {
        if self.failed > 0 {
            if can_mutate {
                "Next action: open the failed job log, compare the exit code with the script, download any artifacts, then retry only the failed stage after fixing the cause."
            } else {
                "Next action: open the failed job log and artifacts, then ask a CI maintainer to retry after the cause is fixed."
            }
        } else if self.blocked > 0 {
            "Next action: inspect the blocked jobs' Needs list and upstream stages. A blocked job usually waits for a dependency to finish or for a failed dependency to be retried."
        } else if self.pending > 0 && self.running == 0 {
            "Next action: check runner health and tag matching. Queued jobs stay pending when no active runner can claim their tags."
        } else if pipeline.state == State::Running || self.running > 0 {
            "Next action: watch the active job trace and reload artifacts after the job finishes."
        } else if pipeline.state == State::Succeeded {
            "Pipeline completed. Review artifacts from the producing jobs before promoting the commit."
        } else if pipeline.state == State::Cancelled {
            if can_mutate {
                "Pipeline was cancelled. Retry it if the ref and CI config are still valid."
            } else {
                "Pipeline was cancelled. Ask a CI maintainer to retry it if needed."
            }
        } else {
            "No blocking diagnosis yet. Refresh pipeline state after runners process the config."
        }
    }
}
pub(super) fn activity(rows: &[Pipeline]) -> &'static str {
    match rows.first().map(|row| row.state) {
        None => {
            "No pipeline history yet. Create a manual pipeline or push a commit to start collecting CI diagnostics."
        }
        Some(State::Failed) => {
            "Latest pipeline failed. Select it to inspect failed jobs, trace output, artifacts, and retry options."
        }
        Some(State::Running) => {
            "Latest pipeline is running. Watch active job traces and refresh artifacts after completion."
        }
        Some(State::Pending | State::Blocked) => {
            "Latest pipeline is queued. Check runner health and tag matching if it does not start soon."
        }
        Some(State::Succeeded) => {
            "Latest pipeline succeeded. Use job artifacts and logs as the release evidence trail."
        }
        Some(State::Cancelled) => {
            "Latest pipeline was cancelled. Retry it only if the ref and CI config still represent the intended run."
        }
    }
}
pub(super) fn description(state: State) -> &'static str {
    match state {
        State::Pending => "Waiting for dependencies or an available runner.",
        State::Blocked => "Blocked by upstream job dependencies.",
        State::Running => "Currently executing on a runner.",
        State::Succeeded => "Completed successfully.",
        State::Failed => "Failed and needs log review.",
        State::Cancelled => "Cancelled before completion.",
    }
}
pub(super) fn next_job(job: &Job) -> &'static str {
    if job.state == State::Failed {
        if job.artifacts.is_empty() {
            "Next action: inspect the trace around the first failing command. No artifact pattern is declared for this job."
        } else {
            "Next action: inspect the trace around the first failing command, then download artifacts for reports, coverage, screenshots, or build output."
        }
    } else if !job.needs.is_empty() {
        "Next action: check upstream Needs jobs first. This job will remain blocked until required dependencies complete successfully."
    } else {
        "Next action: refresh pipeline state. If it stays blocked, check runner capacity and tag matching."
    }
}
pub(super) fn empty_trace(state: State) -> &'static str {
    match state {
        State::Pending | State::Blocked => {
            "Trace is empty because the job has not run yet. Check dependencies, runner availability, and matching tags."
        }
        State::Running => {
            "Trace may still be streaming. Reload the job after the runner flushes output."
        }
        _ => {
            "No trace was stored for this job. Use the last error, exit code, and artifacts to continue diagnosis."
        }
    }
}
pub(super) fn empty_artifacts(job: &Job) -> &'static str {
    if job.artifacts.is_empty() {
        "This job does not declare artifact paths in the CI config, so no downloadable files are expected."
    } else if job.state == State::Failed {
        "Artifact paths were declared, but nothing is available. Check whether the script failed before producing files or whether upload failed."
    } else {
        "Artifact paths were declared, but no files have been uploaded yet. Reload after the job completes."
    }
}
