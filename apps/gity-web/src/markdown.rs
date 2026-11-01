use leptos::prelude::*;
use pulldown_cmark::{Options, Parser, html};
#[component]
pub fn Markdown(content: String) -> impl IntoView {
    let safe = render(&content);
    view! { <article class="markdown-body" inner_html=safe></article> }
}

pub fn render(content: &str) -> String {
    let parser = Parser::new_ext(
        content,
        Options::ENABLE_TABLES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_FOOTNOTES,
    );
    let mut output = String::new();
    html::push_html(&mut output, parser);
    ammonia::Builder::default()
        .add_tag_attributes("code", &["class"])
        .add_tags(&["input"])
        .add_tag_attributes("input", &["checked"])
        .set_tag_attribute_value("input", "type", "checkbox")
        .set_tag_attribute_value("input", "disabled", "")
        .clean(&output)
        .to_string()
}
#[cfg(test)]
#[path = "tests/markdown.rs"]
mod tests;
