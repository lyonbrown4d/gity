use gity_projects::repository_templates::{
  RepositoryTemplateOptions, gitignore_templates, template_files,
};

fn options(gitignore: Option<&'static str>) -> RepositoryTemplateOptions<'static> {
  RepositoryTemplateOptions {
    name: "demo",
    description: None,
    license: None,
    gitignore,
    readme: false,
    year: 2026,
    copyright_holder: "Example Org",
  }
}

#[test]
fn exposes_all_thirteen_catalog_choices() {
  let choices = gitignore_templates();
  assert_eq!(choices.len(), 13);
  assert_eq!(
    choices.iter().map(|choice| choice.id).collect::<Vec<_>>(),
    [
      "rust",
      "node",
      "python",
      "go",
      "java",
      "cpp",
      "visual-studio",
      "dart",
      "swift",
      "kotlin",
      "ruby",
      "unity",
      "android",
    ]
  );
}

#[test]
fn every_choice_returns_its_fixed_root_gitignore_asset() {
  let assets = [
    (
      "rust",
      include_str!("../templates/gitignores/Rust.gitignore"),
    ),
    (
      "node",
      include_str!("../templates/gitignores/Node.gitignore"),
    ),
    (
      "python",
      include_str!("../templates/gitignores/Python.gitignore"),
    ),
    ("go", include_str!("../templates/gitignores/Go.gitignore")),
    (
      "java",
      include_str!("../templates/gitignores/Java.gitignore"),
    ),
    ("cpp", include_str!("../templates/gitignores/C++.gitignore")),
    (
      "visual-studio",
      include_str!("../templates/gitignores/VisualStudio.gitignore"),
    ),
    (
      "dart",
      include_str!("../templates/gitignores/Dart.gitignore"),
    ),
    (
      "swift",
      include_str!("../templates/gitignores/Swift.gitignore"),
    ),
    (
      "kotlin",
      include_str!("../templates/gitignores/Kotlin.gitignore"),
    ),
    (
      "ruby",
      include_str!("../templates/gitignores/Ruby.gitignore"),
    ),
    (
      "unity",
      include_str!("../templates/gitignores/Unity.gitignore"),
    ),
    (
      "android",
      include_str!("../templates/gitignores/Android.gitignore"),
    ),
  ];
  for (id, expected) in assets {
    let files = template_files(options(Some(id))).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, ".gitignore");
    assert_eq!(files[0].content, expected);
  }
}

#[test]
fn rust_and_node_templates_match_their_canonical_assets_byte_for_byte() {
  let rust = template_files(options(Some("rust"))).unwrap();
  let node = template_files(options(Some("node"))).unwrap();
  assert_eq!(
    rust[0].content.as_bytes(),
    include_bytes!("../templates/gitignores/Rust.gitignore")
  );
  assert_eq!(
    node[0].content.as_bytes(),
    include_bytes!("../templates/gitignores/Node.gitignore")
  );
}

#[test]
fn unknown_gitignore_is_a_bad_request() {
  let error = template_files(options(Some("not-a-template"))).unwrap_err();
  assert_eq!(error.status.as_u16(), 400);
}
