use crate::api;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde_json::{Value, json};

pub(super) fn versions(value: &Value) -> Vec<Value> {
  api::list(&value["versions"])
}
pub(super) fn file_count(value: &Value) -> usize {
  versions(value)
    .iter()
    .map(|v| api::list(&v["files"]).len())
    .sum()
}
pub(super) fn bytes(value: u64) -> String {
  if value < 1024 {
    format!("{value} B")
  } else if value < 1024 * 1024 {
    format!("{:.1} KiB", value as f64 / 1024.0)
  } else {
    format!("{:.1} MiB", value as f64 / (1024.0 * 1024.0))
  }
}
pub(super) struct UploadInput {
  pub kind: String,
  pub name: String,
  pub version: String,
  pub filename: String,
  pub path: String,
  pub content_type: String,
  pub inline: String,
  pub encoded: String,
}
impl UploadInput {
  pub fn body(self) -> Result<Value, &'static str> {
    let name = self.name.trim();
    let version = self.version.trim();
    let filename = self.filename.trim();
    let encoded = if self.encoded.is_empty() {
      STANDARD.encode(self.inline.as_bytes())
    } else {
      self.encoded
    };
    if name.is_empty() || version.is_empty() || filename.is_empty() || encoded.is_empty() {
      return Err("Package name, version, file name, and content are required");
    }
    Ok(
      json!({"type":self.kind,"name":name,"version":version,"file_name":filename,
            "file_path":if self.path.trim().is_empty(){filename}else{self.path.trim()},
            "content_type":self.content_type.trim(),"content_base64":encoded}),
    )
  }
}
#[cfg(test)]
#[path = "../../tests/packages.rs"]
mod tests;
