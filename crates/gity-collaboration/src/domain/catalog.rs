use super::Change;
use serde::Deserialize;
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct LabelCreate {
  pub title: String,
  pub color: String,
  pub description: Option<String>,
}
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct LabelChanges {
  pub title: Change<String>,
  pub color: Change<String>,
  pub description: Change<Option<String>>,
}
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct MilestoneCreate {
  pub title: String,
  pub description: Option<String>,
  pub state: String,
  pub start_date: Option<chrono::NaiveDate>,
  pub due_date: Option<chrono::NaiveDate>,
}
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct MilestoneChanges {
  pub title: Change<String>,
  pub description: Change<Option<String>>,
  pub state: Change<String>,
  pub start_date: Change<Option<chrono::NaiveDate>>,
  pub due_date: Change<Option<chrono::NaiveDate>>,
}
