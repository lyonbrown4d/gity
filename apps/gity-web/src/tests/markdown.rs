use super::render;
#[test]
fn renders_readme_headings_code_and_task_lists() {
  let html = render("# Project\n\n- [x] shipped\n\n```rust\nfn main() {}\n```\n");
  assert!(html.contains("<h1>Project</h1>"));
  assert!(html.contains("language-rust"));
  assert!(html.contains("fn main() {}"));
  assert!(html.contains("type=\"checkbox\""));
  assert!(html.contains("disabled=\"\""));
}
#[test]
fn strips_scripts_and_javascript_links_from_user_content() {
  let html = render(
    "<script>alert(1)</script>\n\n[click](javascript:alert(1))\n\n<img src=x onerror=alert(1)>",
  );
  assert!(!html.contains("<script"));
  assert!(!html.contains("javascript:"));
  assert!(!html.contains("onerror"));
}

#[test]
fn only_read_only_checkboxes_survive_html_input_content() {
  let html = render("<input type=password name=secret onclick=attack()>");
  assert!(html.contains("type=\"checkbox\""));
  assert!(html.contains("disabled=\"\""));
  assert!(!html.contains("password") && !html.contains("onclick") && !html.contains("secret"));
}
