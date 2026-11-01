use leptos::prelude::*;
use regex::{Captures, Regex};
use std::sync::LazyLock;

static REFERENCES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(^|[\s(])#([0-9]+)(?-u:\b)").expect("valid issue reference pattern")
});

pub(super) fn link_references(content: &str, base: &str) -> String {
    REFERENCES
        .replace_all(content, |capture: &Captures<'_>| {
            let Ok(number) = capture[2].parse::<u64>() else {
                return capture[0].to_string();
            };
            if number == 0 {
                return capture[0].to_string();
            }
            format!("{}[#{number}]({base}/issues/{number})", &capture[1])
        })
        .into_owned()
}

#[component]
pub(super) fn IssueMarkdown(content: String, base: String) -> impl IntoView {
    let content = if content.is_empty() {
        "*No content*"
    } else {
        &content
    };
    view! { <crate::markdown::Markdown content=link_references(content, &base) /> }
}

#[cfg(test)]
#[path = "../../tests/issue_markdown.rs"]
mod tests;
