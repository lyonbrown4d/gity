pub struct RefInfo {
  pub name: String,
  pub oid: git2::Oid,
}

/// 列出所有 refs（包括 heads/tags）
pub fn list_refs(repo: &git2::Repository) -> Result<Vec<RefInfo>, git2::Error> {
  let mut out = Vec::new();
  let refs = repo.references()?;
  for item in refs {
    let reference = item?;
    let Some(name) = reference.name() else {
      continue;
    };
    let Some(oid) = reference.target() else {
      continue;
    };
    out.push(RefInfo {
      name: name.to_string(),
      oid,
    });
  }
  Ok(out)
}
