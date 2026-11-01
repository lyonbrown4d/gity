use utoipa::IntoParams;
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct ProjectPath {
  pub project: i64,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct PipelinePath {
  pub project: i64,
  pub pipeline: i64,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct JobPath {
  pub project: i64,
  pub job: i64,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct RunnerPath {
  pub project: i64,
  pub runner: i64,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct ArtifactPath {
  pub project: i64,
  pub job: i64,
  pub artifact: i64,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct AgentJobPath {
  pub job: i64,
}
