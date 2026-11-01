pub fn language(path: &str) -> String {
  match path.rsplit('.').next().unwrap_or("") {
    "rs" => "rust",
    "ts" => "typescript",
    "js" => "javascript",
    "json" => "json",
    "yml" | "yaml" => "yaml",
    "md" => "markdown",
    "html" => "html",
    "css" => "css",
    "py" => "python",
    "go" => "go",
    _ => "plaintext",
  }
  .into()
}
