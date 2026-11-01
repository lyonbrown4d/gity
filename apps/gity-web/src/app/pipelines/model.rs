use crate::api;
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum State {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Blocked,
}
impl State {
    pub fn parse(value: &str) -> Self {
        match value {
            "running" => Self::Running,
            "succeeded" => Self::Succeeded,
            "failed" => Self::Failed,
            "canceled" | "cancelled" => Self::Cancelled,
            "blocked" => Self::Blocked,
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
            Self::Blocked => "blocked",
        }
    }
    pub fn terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }
    pub fn retry_job(self) -> bool {
        matches!(self, Self::Failed | Self::Cancelled)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Pipeline {
    pub id: String,
    pub iid: i64,
    pub name: String,
    pub source: String,
    pub reference: String,
    pub commit: String,
    pub state: State,
    pub config: String,
    pub updated: String,
    pub started: String,
    pub finished: String,
}
impl From<&Value> for Pipeline {
    fn from(value: &Value) -> Self {
        Self {
            id: api::text(value, "id"),
            iid: value["iid"].as_i64().unwrap_or_default(),
            name: api::text(value, "name"),
            source: api::text(value, "source"),
            reference: api::text(value, "ref_name"),
            commit: api::text(value, "commit_sha"),
            state: State::parse(&api::text(value, "status")),
            config: api::text(value, "config_source"),
            updated: api::text(value, "updated_at"),
            started: timestamp(&api::text(value, "started_at")),
            finished: timestamp(&api::text(value, "finished_at")),
        }
    }
}
impl Pipeline {
    pub fn title(&self) -> String {
        format!(
            "#{} {}",
            self.iid,
            if self.name.is_empty() {
                "pipeline"
            } else {
                &self.name
            }
        )
    }
    pub fn short_commit(&self) -> String {
        if self.commit.is_empty() {
            "unknown".into()
        } else {
            self.commit.chars().take(8).collect()
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Job {
    pub id: String,
    pub name: String,
    pub stage: String,
    pub state: State,
    pub underlying: State,
    pub attempts: String,
    pub max_attempts: String,
    pub worker: String,
    pub updated: String,
    pub error: String,
    pub needs: Vec<String>,
    pub script: Vec<String>,
    pub artifacts: Vec<String>,
    pub tags: Vec<String>,
}
impl From<&Value> for Job {
    fn from(value: &Value) -> Self {
        let job = &value["project_job"];
        let link = &value["pipeline_job"];
        Self {
            id: api::text(job, "id"),
            name: api::text(link, "name"),
            stage: api::text(link, "stage"),
            state: State::parse(&api::text(value, "status")),
            underlying: State::parse(&api::text(job, "status")),
            attempts: api::text(job, "attempts"),
            max_attempts: api::text(job, "max_attempts"),
            worker: api::text(job, "locked_by"),
            updated: api::text(job, "updated_at"),
            error: api::text(job, "last_error"),
            needs: strings(&value["needs"]),
            script: strings(&value["script"]),
            artifacts: strings(&value["artifacts"]),
            tags: strings(&value["tags"]),
        }
    }
}
impl Job {
    pub fn title(&self) -> String {
        if !self.name.is_empty() {
            self.name.clone()
        } else if !self.stage.is_empty() {
            self.stage.clone()
        } else {
            "job".into()
        }
    }
}
pub(super) fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}
pub(super) fn timestamp(value: &str) -> String {
    if value.starts_with("0001-") {
        String::new()
    } else {
        value.into()
    }
}
pub(super) fn pipelines(value: &Value) -> Vec<Pipeline> {
    let mut rows = value
        .as_array()
        .into_iter()
        .flatten()
        .map(Pipeline::from)
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| std::cmp::Reverse(row.iid));
    rows
}
pub(super) fn create(
    source: &str,
    reference: &str,
    commit: &str,
    config: &str,
    content: &str,
) -> Result<Value, &'static str> {
    if !matches!(source, "web" | "manual" | "merge_request") {
        return Err("Push pipelines are created automatically by accepted Git updates.");
    }
    if reference.trim().is_empty() {
        return Err("Ref is required");
    }
    if content.trim().is_empty() {
        return Err("CI config content is required");
    }
    Ok(
        json!({"source":source,"ref_name":reference.trim(),"commit_sha":commit.trim(),"config_source":config.trim(),"config_content":content}),
    )
}
pub(super) fn trace(value: &Value, selected: &str) -> Option<(String, i64, i64)> {
    if api::text(&value["job"], "id") != selected {
        return None;
    }
    let log = &value["log"];
    let output = value["output"]
        .as_str()
        .or_else(|| log["output"].as_str())
        .unwrap_or_default()
        .to_string();
    Some((
        output,
        log["duration_millis"].as_i64().unwrap_or_default(),
        log["exit_code"].as_i64().unwrap_or_default(),
    ))
}
pub(super) fn bytes(size: u64) -> String {
    if size < 1024 {
        return format!("{size} B");
    }
    let units = ["KB", "MB", "GB", "TB"];
    let mut value = size as f64 / 1024.;
    let mut index = 0;
    while value >= 1024. && index < units.len() - 1 {
        value /= 1024.;
        index += 1;
    }
    format!("{value:.1} {}", units[index])
}
#[cfg(test)]
#[path = "model_tests.rs"]
mod model_tests;
