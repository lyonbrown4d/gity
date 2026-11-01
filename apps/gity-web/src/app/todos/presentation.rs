use crate::api;
use serde_json::Value;

pub(super) fn action_url(todo: &Value) -> String {
    let action = api::text(todo, "action_url");
    if action.starts_with("/app/") {
        action
    } else {
        format!(
            "/app/projects/{}/{}",
            api::text(todo, "organization_id"),
            api::text(todo, "project_id")
        )
    }
}

pub(super) fn format_kind(kind: &str) -> String {
    kind.split('_')
        .map(|word| {
            let mut letters = word.chars();
            letters
                .next()
                .map(|first| first.to_uppercase().to_string() + letters.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn format_time(value: &str) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(value));
    if !date.get_time().is_finite() {
        return value.into();
    }
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let hour = date.get_hours();
    format!(
        "{} {}, {} {}:{:02} {}",
        months[date.get_month() as usize],
        date.get_date(),
        date.get_full_year(),
        match hour % 12 {
            0 => 12,
            n => n,
        },
        date.get_minutes(),
        if hour >= 12 { "PM" } else { "AM" }
    )
}

#[cfg(test)]
#[path = "../../tests/todo_presentation.rs"]
mod tests;
