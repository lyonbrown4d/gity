use crate::{commit, GitService, SearchPlan};
use anyhow::{bail, Result};
use git2::{ObjectType, Repository, Tree};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Clone, Default, Deserialize)]
#[serde(default)]
pub struct SearchParams {
    pub query: String,
    pub path: String,
    pub limit: usize,
    pub max_files: usize,
    pub max_file_size: usize,
    pub match_case: bool,
    pub regex: bool,
}
impl GitService {
    pub async fn resolve_revision(
        &self,
        repo: &str,
        revision: &str,
        default: &str,
    ) -> Result<String> {
        let (revision, default) = (revision.to_owned(), default.to_owned());
        let result = self
            .read(repo, move |r| {
                Ok(json!(commit(&r, &revision, &default)?.id().to_string()))
            })
            .await?;
        Ok(result.as_str().unwrap_or_default().to_owned())
    }

    pub async fn search(
        &self,
        repo: &str,
        revision: &str,
        default: &str,
        input: SearchParams,
    ) -> Result<Value> {
        let plan = SearchPlan::new(&input)?;
        let (revision, default) = (revision.to_owned(), default.to_owned());
        self.read(repo, move |r| {
            if crate::is_empty(&r)? {
                return Ok(json!([]));
            }
            let tree = commit(&r, &revision, &default)?.tree()?;
            let mut count = 0;
            let mut results = Vec::new();
            visit(&r, &tree, "", plan.max_file_size, &mut |path, blob| {
                if results.len() >= plan.limit || count >= plan.max_files {
                    return false;
                }
                if !plan.in_scope(path) {
                    return true;
                }
                count += 1;
                plan.append_matches(path, blob.content(), &mut results);
                true
            })?;
            Ok(json!(results))
        })
        .await
    }

    pub async fn analyze_languages(
        &self,
        repo: &str,
        revision: &str,
        default: &str,
    ) -> Result<Value> {
        let (revision, default) = (revision.to_owned(), default.to_owned());
        self.read(repo, move |r| {
            if crate::is_empty(&r)? {
                return Ok(json!({"revision":"","total_bytes":0,"languages":[]}));
            }
            let c = commit(&r, &revision, &default)?;
            let mut languages = BTreeMap::<String, usize>::new();
            visit(&r, &c.tree()?, "", 10 * 1024 * 1024, &mut |path, blob| {
                if let Some(language) = language(path, blob.content()) {
                    *languages.entry(language.to_owned()).or_default() += blob.size();
                }
                true
            })?;
            let total: usize = languages.values().sum();
            let mut stats = languages
                .into_iter()
                .map(|(language, bytes)| {
                    json!({
                                "language":language,
                                "bytes":bytes,
                                "percentage":if total==0{
                                    0.0
                                }
                                else{
                                    bytes as f64*100.0/total as f64
                                }

                            }
                    )
                })
                .collect::<Vec<_>>();
            stats.sort_by(|a, b| {
                b["bytes"]
                    .as_u64()
                    .cmp(&a["bytes"].as_u64())
                    .then_with(|| a["language"].as_str().cmp(&b["language"].as_str()))
            });
            Ok(json!({"revision":c.id().to_string(),"total_bytes":total,"languages":stats}))
        })
        .await
    }
}
fn visit(
    repo: &Repository,
    tree: &Tree<'_>,
    prefix: &str,
    maximum_size: usize,
    visitor: &mut impl FnMut(&str, &git2::Blob<'_>) -> bool,
) -> Result<bool> {
    if prefix.split('/').count() > 256 {
        bail!("Git tree exceeds supported nesting depth");
    }
    for entry in tree.iter() {
        let name = entry.name()?;
        let path = if prefix.is_empty() {
            name.to_owned()
        } else {
            format!("{prefix}/{name}")
        };
        match entry.kind() {
            Some(ObjectType::Tree) => {
                if !visit(
                    repo,
                    &repo.find_tree(entry.id())?,
                    &path,
                    maximum_size,
                    visitor,
                )? {
                    return Ok(false);
                }
            }
            Some(ObjectType::Blob) => {
                if repo.odb()?.read_header(entry.id())?.0 > maximum_size {
                    continue;
                }
                if !visitor(&path, &repo.find_blob(entry.id())?) {
                    return Ok(false);
                }
            }
            _ => {}
        }
    }
    Ok(true)
}
// Deterministic filename/shebang detection; vendor and generated files are excluded.
fn language(path: &str, content: &[u8]) -> Option<&'static str> {
    if content.contains(&0)
        || path
            .split('/')
            .any(|p| ["vendor", "node_modules", "target", "dist", "build"].contains(&p))
        || path.ends_with(".min.js")
        || path.ends_with(".min.css")
    {
        return None;
    }
    let text = String::from_utf8_lossy(&content[..content.len().min(2048)]);
    if text.contains("Code generated") || text.contains("DO NOT EDIT") {
        return None;
    }
    let name = path.rsplit('/').next()?;
    if name == "Dockerfile" {
        return Some("Dockerfile");
    }
    if name == "Makefile" {
        return Some("Makefile");
    }
    let ext = name.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("");
    Some(match ext {
        "rs" => "Rust",
        "go" => "Go",
        "java" => "Java",
        "kt" | "kts" => "Kotlin",
        "py" => "Python",
        "js" | "jsx" | "mjs" | "cjs" => "JavaScript",
        "ts" | "tsx" => "TypeScript",
        "html" | "htm" => "HTML",
        "css" => "CSS",
        "scss" => "SCSS",
        "sass" => "Sass",
        "less" => "Less",
        "vue" => "Vue",
        "svelte" => "Svelte",
        "c" | "h" => "C",
        "cpp" | "cc" | "cxx" | "hpp" => "C++",
        "cs" => "C#",
        "rb" => "Ruby",
        "php" => "PHP",
        "swift" => "Swift",
        "sh" | "bash" => "Shell",
        "sql" => "SQL",
        "lua" => "Lua",
        "r" | "R" => "R",
        "dart" => "Dart",
        "ex" | "exs" => "Elixir",
        "erl" => "Erlang",
        "scala" => "Scala",
        "pl" => "Perl",
        "proto" => "Protocol Buffer",
        "tf" => "HCL",
        "nix" => "Nix",
        "zig" => "Zig",
        "hs" => "Haskell",
        _ if text.starts_with("#!/usr/bin/env python") || text.starts_with("#!/usr/bin/python") => {
            "Python"
        }
        _ if text.starts_with("#!/bin/sh") || text.starts_with("#!/bin/bash") => "Shell",
        _ => return None,
    })
}

/// Visit blobs without retaining repository handles across asynchronous boundaries.
/// Large objects are rejected by ODB metadata before their contents are loaded.
pub fn walk_blobs(
    repo: &Repository,
    tree: &Tree<'_>,
    maximum_size: usize,
    visitor: &mut impl FnMut(&str, &git2::Blob<'_>) -> bool,
) -> Result<bool> {
    visit(repo, tree, "", maximum_size, visitor)
}
