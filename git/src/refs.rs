use gix::prelude::ObjectIdExt;

pub struct RefInfo {
  pub name: String,
  pub oid: gix::hash::ObjectId,
}

/// 列出所有 refs（包括 heads/tags）
pub fn list_refs(repo: &gix::Repository) -> Vec<RefInfo> {
  let mut out = Vec::new();
  if let Ok(store) = repo.references() {
    // for reference in store {
    //     if let Ok(r) = reference {
    //         if let Some(id) = r.target.try_id() {
    //             out.push(RefInfo {
    //                 name: r.name.to_string(),
    //                 oid: id,
    //             });
    //         }
    //     }
    // }
  }
  out
}
