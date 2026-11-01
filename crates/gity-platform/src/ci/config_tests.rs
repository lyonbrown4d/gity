use super::*;
#[test]
fn yaml_stages_dependency_and_script_validation() {
  let config=parse("stages: [build, test]\nvariables: {MODE: test}\njobs:\n  compile: {stage: build, script: 'echo build'}\n  test: {script: ['echo test'], artifacts: {paths: [report.txt]}}\n").unwrap();
  assert_eq!(config.jobs["test"].needs, Some(vec!["compile".into()]));
  for yaml in [
    "jobs: {a: {script: x, needs: [b]}, b: {script: x, needs: [a]}}",
    "jobs: {a: {script: []}}",
    "jobs: {a: {script: x, artifacts: ['../secret']}}",
    "pipeline { stage test {} }",
    "jobs: {a: {script: x, invented: true}}",
  ] {
    assert!(parse(yaml).is_err(), "{yaml}");
  }
}
