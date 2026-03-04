use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct Page<T> {
  pub total: u64,
  pub page: u64,
  pub page_size: u64,
  pub items: Vec<T>,
}
