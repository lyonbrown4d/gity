pub(super) use crate::ui::relative_time;
use crate::{api, ui::Query};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
pub(super) struct IssueQueries {
    pub issue: Query,
    pub comments: Query,
    pub assignees: Query,
    pub labels: Query,
    pub users: Query,
    pub permissions: Query,
}

pub(super) fn user_label(users: &Value, id: &str) -> String {
    if let Some(user) = api::list(users)
        .into_iter()
        .find(|user| api::text(user, "id") == id)
    {
        let username = api::text(&user, "username");
        let display_name = api::text(&user, "display_name");
        if display_name.trim().is_empty() {
            format!("@{username}")
        } else {
            format!("{} (@{username})", display_name.trim())
        }
    } else if id.trim().is_empty() {
        "Unknown user".into()
    } else {
        format!("#{id}")
    }
}

pub(super) fn labels_body(
    rows: Vec<Value>,
    remove: Option<&str>,
    addition: Option<(&str, &str)>,
) -> Value {
    let mut seen = BTreeSet::new();
    let mut labels = rows
        .into_iter()
        .filter(|row| remove != Some(api::text(row, "name").as_str()))
        .map(|row| (api::text(&row, "name"), api::text(&row, "color")))
        .collect::<Vec<_>>();
    if let Some((name, color)) = addition {
        labels.push((name.to_string(), color.to_string()));
    }
    json!({"labels":labels.into_iter().filter_map(|(name,color)| {
        let name = name.trim().to_string();
        if name.is_empty() || !seen.insert(name.clone()) { None } else { Some(json!({"name":name,"color":color.trim()})) }
    }).collect::<Vec<_>>()})
}

#[cfg(test)]
#[path = "../../tests/issue_detail.rs"]
mod tests;
