//! Business lifecycle rules shared by every collaboration adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueState {
  Opened,
  Closed,
}
impl IssueState {
  pub fn parse(value: &str) -> Result<Self, &'static str> {
    match value {
      "opened" => Ok(Self::Opened),
      "closed" => Ok(Self::Closed),
      _ => Err("issue state must be opened or closed"),
    }
  }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MergeRequestState {
  Opened,
  Closed,
  Merged,
}
impl MergeRequestState {
  pub fn editable_transition(current: &str, next: &str) -> Result<Self, &'static str> {
    let next = match next {
      "opened" => Self::Opened,
      "closed" => Self::Closed,
      _ => return Err("use merge action to merge a merge request"),
    };
    if current == "merged" {
      return Err("merged merge requests cannot be reopened");
    }
    Ok(next)
  }
}
pub fn nonempty(value: &str) -> bool {
  !value.trim().is_empty()
}
