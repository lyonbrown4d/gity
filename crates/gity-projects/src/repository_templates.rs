//! Compile-time embedded starter files for newly initialized repositories.
use gity_core::AppError;
use serde::Serialize;
mod gitignores;
pub use gitignores::{GitignoreTemplate, gitignore_templates};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct LicenseTemplate {
  pub id: &'static str,
  pub name: &'static str,
  pub spdx_id: &'static str,
}

const LICENSES: &[LicenseTemplate] = &[
  LicenseTemplate {
    id: "mit",
    name: "MIT License",
    spdx_id: "MIT",
  },
  LicenseTemplate {
    id: "apache-2.0",
    name: "Apache License 2.0",
    spdx_id: "Apache-2.0",
  },
  LicenseTemplate {
    id: "gpl-3.0-only",
    name: "GNU General Public License v3.0 only",
    spdx_id: "GPL-3.0-only",
  },
  LicenseTemplate {
    id: "agpl-3.0-only",
    name: "GNU Affero General Public License v3.0 only",
    spdx_id: "AGPL-3.0-only",
  },
  LicenseTemplate {
    id: "bsd-2-clause",
    name: "BSD 2-Clause License",
    spdx_id: "BSD-2-Clause",
  },
  LicenseTemplate {
    id: "bsd-3-clause",
    name: "BSD 3-Clause License",
    spdx_id: "BSD-3-Clause",
  },
  LicenseTemplate {
    id: "mpl-2.0",
    name: "Mozilla Public License 2.0",
    spdx_id: "MPL-2.0",
  },
  LicenseTemplate {
    id: "unlicense",
    name: "The Unlicense",
    spdx_id: "Unlicense",
  },
];

pub fn license_templates() -> &'static [LicenseTemplate] {
  LICENSES
}

pub struct RepositoryTemplateOptions<'a> {
  pub name: &'a str,
  pub description: Option<&'a str>,
  pub license: Option<&'a str>,
  pub gitignore: Option<&'a str>,
  pub readme: bool,
  pub year: i32,
  pub copyright_holder: &'a str,
}

#[derive(Debug)]
pub struct TemplateFile {
  pub path: String,
  pub content: String,
}

pub fn template_files(
  options: RepositoryTemplateOptions<'_>,
) -> Result<Vec<TemplateFile>, AppError> {
  let mut files = Vec::new();
  if options.readme {
    files.push(TemplateFile {
      path: "README.md".into(),
      content: readme(&options),
    });
  }
  if let Some(license) = selected(options.license) {
    if options.copyright_holder.trim().is_empty()
      || options.copyright_holder.chars().count() > 200
      || options.copyright_holder.chars().any(char::is_control)
    {
      return Err(AppError::bad_request("invalid license copyright holder"));
    }
    let (text, filename) = license_text(license)?;
    let mut content = text.to_owned();
    if matches!(license, "mit" | "bsd-2-clause" | "bsd-3-clause") {
      content = content.replace("<year>", &options.year.to_string());
      content = content
        .replace("<copyright holders>", options.copyright_holder)
        .replace("<owner>", options.copyright_holder);
    }
    files.push(TemplateFile {
      path: filename.into(),
      content,
    });
  }
  if let Some(template) = selected(options.gitignore) {
    let content = gitignores::template_content(template)?;
    files.push(TemplateFile {
      path: ".gitignore".into(),
      content: content.into(),
    });
  }
  Ok(files)
}

fn selected(value: Option<&str>) -> Option<&str> {
  value.filter(|value| !value.trim().is_empty() && *value != "none")
}

fn license_text(id: &str) -> Result<(&'static str, &'static str), AppError> {
  let (text, filename) = match id {
    "mit" => (include_str!("../templates/MIT.txt"), "LICENSE"),
    "apache-2.0" => (include_str!("../templates/Apache-2.0.txt"), "LICENSE"),
    "gpl-3.0-only" | "gpl-3.0" => (include_str!("../templates/GPL-3.0-only.txt"), "LICENSE"),
    "agpl-3.0-only" => (include_str!("../templates/AGPL-3.0-only.txt"), "LICENSE"),
    "bsd-2-clause" => (include_str!("../templates/BSD-2-Clause.txt"), "LICENSE"),
    "bsd-3-clause" => (include_str!("../templates/BSD-3-Clause.txt"), "LICENSE"),
    "mpl-2.0" => (include_str!("../templates/MPL-2.0.txt"), "LICENSE"),
    "unlicense" => (include_str!("../templates/Unlicense.txt"), "LICENSE"),
    _ => return Err(AppError::bad_request("unknown license template")),
  };
  Ok((text, filename))
}

fn readme(options: &RepositoryTemplateOptions<'_>) -> String {
  let mut result = format!("# {}\n", markdown_plain(options.name));
  if let Some(description) = options.description.filter(|value| !value.trim().is_empty()) {
    result.push('\n');
    result.push_str(&markdown_plain(description));
    result.push('\n');
  }
  result
}

fn markdown_plain(value: &str) -> String {
  let mut escaped = String::with_capacity(value.len());
  for character in value.chars() {
    if character.is_control() {
      if character == '\n' || character == '\r' || character == '\t' {
        escaped.push(' ');
      }
    } else {
      if "\\`*_{}[]<>()#+-.!|&".contains(character) {
        escaped.push('\\');
      }
      escaped.push(character);
    }
  }
  escaped
}
