use crate::http::auth::ErrorResponse;
use axum::Json;
use axum::http::StatusCode;
use domain::page::Page;

#[derive(Debug, Clone, Copy)]
pub struct Pagination {
  pub page: u64,
  pub page_size: u64,
}

pub fn resolve_pagination(
  page: Option<u64>,
  page_size: Option<u64>,
  default_page_size: u64,
  max_page_size: u64,
) -> Result<Pagination, (StatusCode, Json<ErrorResponse>)> {
  let resolved_page = page.unwrap_or(1);
  if resolved_page == 0 {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: "page must be greater than 0".to_string(),
      }),
    ));
  }

  let resolved_page_size = page_size.unwrap_or(default_page_size);
  if resolved_page_size == 0 || resolved_page_size > max_page_size {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        message: format!("page_size must be in range [1, {max_page_size}]"),
      }),
    ));
  }

  Ok(Pagination {
    page: resolved_page,
    page_size: resolved_page_size,
  })
}

pub fn to_page<T>(items: Vec<T>, total: u64, pagination: Pagination) -> Page<T> {
  Page {
    total,
    page: pagination.page,
    page_size: pagination.page_size,
    items,
  }
}

pub fn to_single_page<T>(items: Vec<T>) -> Page<T> {
  let total = items.len() as u64;
  Page {
    total,
    page: 1,
    page_size: total.max(1),
    items,
  }
}
