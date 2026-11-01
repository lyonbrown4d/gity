use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
#[derive(IntoParams)]
#[into_params(parameter_in=Path)]
pub struct GitPath {
  pub organization: String,
  /// Project path key including the required .git suffix.
  pub project: String,
}
#[derive(Serialize, Deserialize, ToSchema)]
pub enum GitService {
  #[serde(rename = "git-upload-pack")]
  UploadPack,
  #[serde(rename = "git-receive-pack")]
  ReceivePack,
}
#[derive(IntoParams)]
#[into_params(parameter_in=Query)]
pub struct DiscoveryQuery {
  #[param(inline)]
  pub service: GitService,
}
