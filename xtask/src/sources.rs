use anyhow::{Result, bail};
use std::path::Path;
pub fn check(root: &Path) -> Result<()> {
  let mut violations = Vec::new();
  for legacy in [
    "cmd",
    "internal",
    "web",
    "go.mod",
    "go.sum",
    ".golangci.yml",
    ".goreleaser.yaml",
    "package.json",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "bun.lock",
    "bun.lockb",
    "vite.config.ts",
    "vite.config.js",
  ] {
    if root.join(legacy).exists() {
      violations.push(format!(
        "{legacy}: retired Go/React implementation must not return"
      ));
    }
  }
  for directory in ["apps", "crates", "xtask", "scripts", "perf"] {
    walk(&root.join(directory), &mut violations)?;
  }
  if !violations.is_empty() {
    bail!("source policy violations:\n{}", violations.join("\n"));
  }
  Ok(())
}
fn walk(path: &Path, violations: &mut Vec<String>) -> Result<()> {
  for entry in std::fs::read_dir(path)? {
    let entry = entry?;
    let path = entry.path();
    let kind = entry.file_type()?;
    let name = entry.file_name().to_string_lossy().into_owned();
    if [
      "package.json",
      "package-lock.json",
      "pnpm-lock.yaml",
      "yarn.lock",
      "bun.lock",
      "bun.lockb",
    ]
    .contains(&name.as_str())
      || path.extension().is_some_and(|ext| {
        ["go", "ts", "tsx", "jsx", "mjs", "plano"]
          .iter()
          .any(|v| ext == *v)
      })
    {
      violations.push(format!(
        "{}: legacy language or Node build input is forbidden",
        path.display()
      ));
    }
    if kind.is_dir() {
      if !["vendor", "target", "dist", "node_modules"]
        .contains(&entry.file_name().to_string_lossy().as_ref())
      {
        walk(&path, violations)?;
      }
      continue;
    }
    if !kind.is_file()
      || !path.extension().is_some_and(|v| {
        ["rs", "css", "html", "json", "js", "mjs", "ps1", "sql"]
          .iter()
          .any(|ext| v == *ext)
      })
    {
      continue;
    }
    let text = std::fs::read_to_string(&path)?;
    let lines = text.lines().count();
    if lines > 300 {
      violations.push(format!("{}: {lines} lines (maximum 300)", path.display()));
    }
    let is_test = path.components().any(|v| v.as_os_str() == "tests")
      || name == "tests.rs"
      || name.ends_with("_tests.rs");
    if path.extension().is_some_and(|v| v == "rs")
      && !is_test
      && text
        .lines()
        .any(|line| line.trim().starts_with("#[test]") || line.trim().starts_with("#[tokio::test"))
    {
      violations.push(format!(
        "{}: tests must live in a separate test file",
        path.display()
      ));
    }
  }
  Ok(())
}
