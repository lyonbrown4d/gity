use utoipa::IntoParams;
#[derive(Default, serde::Deserialize, IntoParams)]
#[into_params(parameter_in=Query)]
pub struct TraceQuery {
  /// Select an attempt (otherwise latest persisted attempt). Enables bounded mode.
  #[param(minimum = 1)]
  pub attempt: Option<i32>,
  /// Byte offset. Offsets inside UTF-8 characters advance to the next boundary.
  pub offset: Option<u64>,
  /// Maximum returned output bytes, 4–262144; bounded mode defaults to 65536.
  #[param(minimum = 4, maximum = 262144)]
  pub limit: Option<u64>,
}
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
