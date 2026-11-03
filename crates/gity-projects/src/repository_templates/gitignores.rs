use gity_core::AppError;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct GitignoreTemplate {
  pub id: &'static str,
  pub name: &'static str,
}

const TEMPLATES: &[GitignoreTemplate] = &[
  GitignoreTemplate {
    id: "rust",
    name: "Rust",
  },
  GitignoreTemplate {
    id: "node",
    name: "Node.js",
  },
  GitignoreTemplate {
    id: "python",
    name: "Python",
  },
  GitignoreTemplate {
    id: "go",
    name: "Go",
  },
  GitignoreTemplate {
    id: "java",
    name: "Java",
  },
  GitignoreTemplate {
    id: "cpp",
    name: "C++",
  },
  GitignoreTemplate {
    id: "visual-studio",
    name: "Visual Studio",
  },
  GitignoreTemplate {
    id: "dart",
    name: "Dart",
  },
  GitignoreTemplate {
    id: "swift",
    name: "Swift",
  },
  GitignoreTemplate {
    id: "kotlin",
    name: "Kotlin",
  },
  GitignoreTemplate {
    id: "ruby",
    name: "Ruby",
  },
  GitignoreTemplate {
    id: "unity",
    name: "Unity",
  },
  GitignoreTemplate {
    id: "android",
    name: "Android",
  },
];

pub fn gitignore_templates() -> &'static [GitignoreTemplate] {
  TEMPLATES
}

pub(super) fn template_content(id: &str) -> Result<&'static str, AppError> {
  match id {
    "rust" => Ok(include_str!("../../templates/gitignores/Rust.gitignore")),
    "node" => Ok(include_str!("../../templates/gitignores/Node.gitignore")),
    "python" => Ok(include_str!("../../templates/gitignores/Python.gitignore")),
    "go" => Ok(include_str!("../../templates/gitignores/Go.gitignore")),
    "java" => Ok(include_str!("../../templates/gitignores/Java.gitignore")),
    "cpp" => Ok(include_str!("../../templates/gitignores/C++.gitignore")),
    "visual-studio" => Ok(include_str!(
      "../../templates/gitignores/VisualStudio.gitignore"
    )),
    "dart" => Ok(include_str!("../../templates/gitignores/Dart.gitignore")),
    "swift" => Ok(include_str!("../../templates/gitignores/Swift.gitignore")),
    "kotlin" => Ok(include_str!("../../templates/gitignores/Kotlin.gitignore")),
    "ruby" => Ok(include_str!("../../templates/gitignores/Ruby.gitignore")),
    "unity" => Ok(include_str!("../../templates/gitignores/Unity.gitignore")),
    "android" => Ok(include_str!("../../templates/gitignores/Android.gitignore")),
    _ => Err(AppError::bad_request("unknown gitignore template")),
  }
}
