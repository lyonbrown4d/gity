use super::schemas::Timestamps;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Serialize, ToSchema)]
pub struct Package {
  pub id: String,
  pub project_id: String,
  pub r#type: String,
  pub name: String,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}
#[derive(Serialize, ToSchema)]
pub struct PackageListPage {
  pub items: Vec<Package>,
  pub page: u64,
  pub per_page: u64,
  pub total: u64,
  pub total_pages: u64,
}
#[derive(Serialize, ToSchema)]
pub struct Version {
  pub id: String,
  pub project_package_id: String,
  pub version: String,
  pub status: String,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}
#[derive(Serialize, ToSchema)]
pub struct PackageFile {
  pub id: String,
  pub project_package_version_id: String,
  pub file_name: String,
  pub file_path: String,
  pub content_type: String,
  pub byte_size: i64,
  pub storage_key: String,
  #[serde(flatten)]
  pub timestamps: Timestamps,
}
#[derive(Serialize, ToSchema)]
pub struct VersionDetail {
  pub version: Version,
  pub files: Vec<PackageFile>,
}
#[derive(Serialize, ToSchema)]
pub struct PackageDetail {
  pub package: Package,
  pub versions: Vec<VersionDetail>,
}
#[derive(Deserialize, ToSchema)]
pub struct CreatePackageFile {
  pub r#type: String,
  pub name: String,
  pub version: String,
  pub file_name: String,
  pub file_path: Option<String>,
  pub content_type: Option<String>,
  pub content_base64: String,
}
#[derive(Serialize, ToSchema)]
pub struct PackageContent {
  pub file: PackageFile,
  pub content_base64: String,
}
#[derive(Deserialize, ToSchema)]
pub struct NpmAttachment {
  pub data: String,
  pub content_type: Option<String>,
}
#[derive(Deserialize, ToSchema)]
pub struct NpmPublish {
  pub name: Option<String>,
  #[serde(rename = "dist-tags")]
  pub dist_tags: Option<std::collections::BTreeMap<String, String>>,
  pub versions: Option<std::collections::BTreeMap<String, serde_json::Value>>,
  #[serde(rename = "_attachments")]
  pub attachments: std::collections::BTreeMap<String, NpmAttachment>,
}
#[derive(Serialize, ToSchema)]
pub struct NpmVersion {
  pub name: String,
  pub version: String,
  pub dist: std::collections::BTreeMap<String, String>,
}
#[derive(Serialize, ToSchema)]
pub struct NpmMetadata {
  pub name: String,
  #[serde(rename = "dist-tags")]
  pub dist_tags: std::collections::BTreeMap<String, String>,
  pub versions: std::collections::BTreeMap<String, NpmVersion>,
  pub uploaded_file_id: Option<String>,
}
#[derive(Serialize, ToSchema)]
pub struct NugetResource {
  #[serde(rename = "@id")]
  pub id: String,
  #[serde(rename = "@type")]
  pub r#type: String,
}
#[derive(Serialize, ToSchema)]
pub struct NugetIndex {
  pub version: String,
  pub resources: Vec<NugetResource>,
}
