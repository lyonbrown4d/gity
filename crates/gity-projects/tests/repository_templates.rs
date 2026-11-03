use gity_projects::repository_templates::{
  RepositoryTemplateOptions, license_templates, template_files,
};

fn options() -> RepositoryTemplateOptions<'static> {
  RepositoryTemplateOptions {
    name: "demo",
    description: None,
    license: None,
    gitignore: None,
    readme: false,
    year: 2026,
    copyright_holder: "Example Org",
  }
}

#[test]
fn no_selection_creates_no_files() {
  assert!(template_files(options()).unwrap().is_empty());
}

#[test]
fn readme_only_escapes_markup_and_flattens_newlines() {
  let files = template_files(RepositoryTemplateOptions {
    name: "<script>alert(1)</script>",
    description: Some("first\n# injected"),
    readme: true,
    ..options()
  })
  .unwrap();
  assert_eq!(files.len(), 1);
  assert_eq!(files[0].path, "README.md");
  assert_eq!(
    files[0].content,
    "# \\<script\\>alert\\(1\\)\\</script\\>\n\nfirst \\# injected\n"
  );
  assert!(!files[0].content.contains("<script>"));
}

#[test]
fn exposes_eight_canonical_spdx_choices_and_keeps_full_license_texts() {
  let choices = license_templates();
  assert_eq!(choices.len(), 8);
  assert_eq!(
    choices.iter().map(|item| item.id).collect::<Vec<_>>(),
    [
      "mit",
      "apache-2.0",
      "gpl-3.0-only",
      "agpl-3.0-only",
      "bsd-2-clause",
      "bsd-3-clause",
      "mpl-2.0",
      "unlicense",
    ]
  );
  for choice in choices {
    let files = template_files(RepositoryTemplateOptions {
      license: Some(choice.id),
      ..options()
    })
    .unwrap();
    assert_eq!(files[0].path, "LICENSE");
    assert!(files[0].content.len() > 200);
    assert!(files[0].content.ends_with('\n'));
  }
  let gpl = template_files(RepositoryTemplateOptions {
    license: Some("gpl-3.0-only"),
    ..options()
  })
  .unwrap();
  assert_eq!(
    gpl[0].content,
    include_str!("../templates/GPL-3.0-only.txt")
  );
}

#[test]
fn legacy_gpl_alias_and_notice_placeholders_are_supported() {
  for id in ["mit", "bsd-2-clause", "bsd-3-clause", "gpl-3.0"] {
    let files = template_files(RepositoryTemplateOptions {
      license: Some(id),
      ..options()
    })
    .unwrap();
    if id != "gpl-3.0" {
      assert!(files[0].content.contains("2026"));
      assert!(files[0].content.contains("Example Org"));
      assert!(!files[0].content.contains("<year>"));
    } else {
      assert_eq!(
        files[0].content,
        include_str!("../templates/GPL-3.0-only.txt")
      );
    }
  }
  let apache = template_files(RepositoryTemplateOptions {
    license: Some("apache-2.0"),
    ..options()
  })
  .unwrap();
  assert_eq!(
    apache[0].content,
    include_str!("../templates/Apache-2.0.txt")
  );
}

#[test]
fn gitignore_and_license_combinations_use_fixed_paths() {
  let files = template_files(RepositoryTemplateOptions {
    license: Some("apache-2.0"),
    gitignore: Some("rust"),
    readme: true,
    ..options()
  })
  .unwrap();
  assert_eq!(
    files
      .iter()
      .map(|file| file.path.as_str())
      .collect::<Vec<_>>(),
    ["README.md", "LICENSE", ".gitignore",]
  );
  assert!(files[2].content.contains("target"));
}

#[test]
fn empty_none_are_absent_and_unknown_templates_are_bad_requests() {
  let files = template_files(RepositoryTemplateOptions {
    license: Some("none"),
    gitignore: Some(""),
    ..options()
  })
  .unwrap();
  assert!(files.is_empty());
  let error = template_files(RepositoryTemplateOptions {
    license: Some("typo"),
    ..options()
  })
  .unwrap_err();
  assert_eq!(error.status.as_u16(), 400);
  let error = template_files(RepositoryTemplateOptions {
    gitignore: Some("unknown"),
    ..options()
  })
  .unwrap_err();
  assert_eq!(error.status.as_u16(), 400);
}

#[test]
fn invalid_copyright_holder_is_rejected() {
  let error = template_files(RepositoryTemplateOptions {
    license: Some("mit"),
    copyright_holder: "bad\nheader",
    ..options()
  })
  .unwrap_err();
  assert_eq!(error.status.as_u16(), 400);
}
