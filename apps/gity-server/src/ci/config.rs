use gity_core::AppError;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use utoipa::ToSchema;

#[derive(Clone, Debug, Default, Deserialize, Serialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PipelineConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub stages: Vec<String>,
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub variables: BTreeMap<String, String>,
    pub jobs: BTreeMap<String, JobConfig>,
}
#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct JobConfig {
    #[serde(default = "default_stage")]
    pub stage: String,
    pub script: Script,
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Missing means all jobs in preceding stages; [] means immediately ready.
    #[serde(default)]
    pub needs: Option<Vec<String>>,
    #[serde(default)]
    pub variables: BTreeMap<String, String>,
    #[serde(default)]
    pub artifacts: Artifacts,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}
#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
#[serde(untagged)]
pub enum Script {
    One(String),
    Many(Vec<String>),
}
impl Script {
    pub fn lines(&self) -> Vec<String> {
        match self {
            Self::One(v) => vec![v.clone()],
            Self::Many(v) => v.clone(),
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
#[serde(untagged)]
pub enum Artifacts {
    List(Vec<String>),
    Paths { paths: Vec<String> },
}
impl Default for Artifacts {
    fn default() -> Self {
        Self::List(vec![])
    }
}
impl Artifacts {
    pub fn paths(&self) -> Vec<String> {
        match self {
            Self::List(v) | Self::Paths { paths: v } => v.clone(),
        }
    }
}
fn default_stage() -> String {
    "test".into()
}
fn default_timeout() -> u64 {
    3600
}
pub fn variable_key(key: &str) -> bool {
    let mut chars = key.bytes();
    chars
        .next()
        .is_some_and(|b| b.is_ascii_uppercase() || b == b'_')
        && chars.all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
        && key.len() <= 128
}
pub fn parse(content: &str) -> Result<PipelineConfig, AppError> {
    if content.len() > 1024 * 1024 {
        return Err(AppError::bad_request("CI configuration exceeds 1 MiB"));
    }
    let mut config: PipelineConfig = serde_yaml::from_str(content)
        .map_err(|e| AppError::bad_request(format!("invalid .gity-ci.yml: {e}")))?;
    if config.jobs.is_empty() || config.jobs.len() > 200 {
        return Err(AppError::bad_request(
            "CI configuration requires 1–200 jobs",
        ));
    }
    if config.stages.is_empty() {
        config.stages = vec!["build".into(), "test".into(), "deploy".into()];
    }
    let stages = config.stages.iter().collect::<BTreeSet<_>>();
    if stages.len() != config.stages.len() || config.stages.iter().any(|s| s.is_empty()) {
        return Err(AppError::bad_request(
            "CI stages must be nonempty and unique",
        ));
    }
    for key in config.variables.keys() {
        if !variable_key(key) {
            return Err(AppError::bad_request("invalid CI variable key"));
        }
    }
    let original = config.jobs.clone();
    for (name, job) in &mut config.jobs {
        if name.is_empty()
            || name.len() > 128
            || job.script.lines().is_empty()
            || job
                .script
                .lines()
                .iter()
                .any(|s| s.trim().is_empty() || s.contains('\0'))
        {
            return Err(AppError::bad_request(
                "each CI job requires a name and nonempty script",
            ));
        }
        let stage = config
            .stages
            .iter()
            .position(|s| *s == job.stage)
            .ok_or_else(|| AppError::bad_request(format!("unknown stage for job {name}")))?;
        if job.timeout_seconds == 0 || job.timeout_seconds > 86400 {
            return Err(AppError::bad_request("job timeout must be 1–86400 seconds"));
        }
        for key in job.variables.keys() {
            if !variable_key(key) {
                return Err(AppError::bad_request("invalid job variable key"));
            }
        }
        for path in job.artifacts.paths() {
            if !safe_relative(&path) {
                return Err(AppError::bad_request(
                    "artifact paths must be relative without traversal or wildcards",
                ));
            }
        }
        if job.image.is_empty() {
            job.image = config.image.clone();
        }
        if job.image.starts_with('-') || job.image.contains('\0') {
            return Err(AppError::bad_request("invalid CI container image"));
        }
        if job.needs.is_none() {
            job.needs = Some(
                original
                    .iter()
                    .filter(|(_, j)| {
                        config
                            .stages
                            .iter()
                            .position(|s| s == &j.stage)
                            .is_some_and(|s| s < stage)
                    })
                    .map(|(name, _)| name.clone())
                    .collect(),
            );
        }
        for need in job.needs.as_ref().unwrap() {
            let dependency = original
                .get(need)
                .ok_or_else(|| AppError::bad_request(format!("unknown dependency {need}")))?;
            if need == name
                || config
                    .stages
                    .iter()
                    .position(|s| s == &dependency.stage)
                    .is_none_or(|s| s > stage)
            {
                return Err(AppError::bad_request(
                    "job dependency must be in the same or an earlier stage",
                ));
            }
        }
    }
    let mut done = BTreeSet::new();
    loop {
        let before = done.len();
        for (name, job) in &config.jobs {
            if job.needs.as_ref().unwrap().iter().all(|v| done.contains(v)) {
                done.insert(name.clone());
            }
        }
        if done.len() == config.jobs.len() {
            break;
        }
        if done.len() == before {
            return Err(AppError::bad_request("cyclic CI job dependencies"));
        }
    }
    Ok(config)
}
pub fn safe_relative(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !path.contains('\0')
        && !path.contains(['*', '?', '['])
        && std::path::Path::new(path)
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_)))
        && !path.split('/').any(|s| s.eq_ignore_ascii_case(".git"))
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
