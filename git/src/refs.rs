
pub struct RefInfo {
  pub name: String,
  pub oid: gix::hash::ObjectId,
}

/// 列出所有 refs（包括 heads/tags）
pub fn list_refs(_repo: &gix::Repository) -> Vec<RefInfo> {
    // placeholder implementation; we no longer depend on gix for this
    // functionality because the HTTP module executes `git` directly.
    Vec::new()
}
