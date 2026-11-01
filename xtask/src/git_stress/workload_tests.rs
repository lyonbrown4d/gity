use super::operations::choose;
use super::process::sanitize;
use crate::git_stress::{Config, Mode};

fn config(mode: Mode) -> Config {
    Config {
        base_url: String::new(),
        workers: 1,
        iterations: 4,
        mode,
        weights: [20, 45, 25, 10],
        timeout_ms: 10,
        workdir: String::new(),
        keep_workdir: false,
        seed_files: 0,
        project_url: String::new(),
        token_env: String::new(),
        token_username: String::new(),
        compose: false,
        build: false,
        project: String::new(),
    }
}

#[test]
fn mixed_workload_selects_each_weighted_operation() {
    let config = config(Mode::Mixed);
    let mut counts = std::collections::BTreeMap::new();
    for iteration in 0..100 {
        *counts.entry(choose(&config, iteration)).or_insert(0) += 1;
    }
    assert_eq!(counts["clone"], 20);
    assert_eq!(counts["fetch"], 45);
    assert_eq!(counts["push"], 25);
    assert_eq!(counts["ls-remote"], 10);
}

#[test]
fn failure_text_redacts_repository_token() {
    assert_eq!(
        sanitize("authentication failed: secret-value", "secret-value"),
        "authentication failed: <token>"
    );
}
