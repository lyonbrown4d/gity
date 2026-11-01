use super::*;
use std::collections::BTreeMap;
#[tokio::test]
async fn shell_runs_commands_and_preserves_failure_status() {
    let temp = tempfile::tempdir().unwrap();
    let work = temp.path().join("work");
    std::fs::create_dir(&work).unwrap();
    let script = temp.path().join("job.sh");
    std::fs::write(&script, "printf '%s' \"$CI_TEST\" > result\nexit 7\n").unwrap();
    let job = Job {
        script: vec!["unused".into()],
        commit_sha: String::new(),
        image: String::new(),
        env: BTreeMap::from([("CI_TEST".into(), "value with spaces and $literal".into())]),
        artifacts: vec![],
        timeout_seconds: 20,
    };
    let options = Options {
        executor: Executor::Shell,
        container_bin: "docker".into(),
        image: "alpine:3.22".into(),
        work_root: temp.path().into(),
        resources: crate::resources::Resources::default(),
    };
    let status = build_command(&job, &options, &work, &script, "test", temp.path())
        .unwrap()
        .status()
        .await
        .unwrap();
    assert_eq!(status.code(), Some(7));
    assert_eq!(
        std::fs::read_to_string(work.join("result")).unwrap(),
        "value with spaces and $literal"
    );
}
