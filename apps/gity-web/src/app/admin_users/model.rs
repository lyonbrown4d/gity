use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, PartialEq, Default)]
pub(super) struct UserPage {
  pub rows: Vec<Value>,
  pub total: u64,
}

#[derive(Deserialize)]
struct Response {
  items: Vec<Value>,
  total: u64,
  page: u64,
  page_size: u64,
}

pub(super) fn user_page(value: &Value) -> Result<UserPage, &'static str> {
  let error = "The user directory could not be loaded. Try again.";
  let response: Response = serde_json::from_value(value.clone()).map_err(|_| error)?;
  if response.page == 0
    || response.page_size == 0
    || response.page_size > 100
    || response.items.len() as u64 > response.page_size
    || response.total < response.items.len() as u64
  {
    return Err(error);
  }
  Ok(UserPage {
    rows: response.items,
    total: response.total,
  })
}

#[cfg(test)]
#[path = "../../tests/admin_users.rs"]
mod tests;
