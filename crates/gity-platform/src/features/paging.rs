//! Shared pagination for collection endpoints.
//!
//! Mirrors the `/users` contract: 1-based pages, 1-100 items, default 20.
use crate::openapi::schemas::ListQuery;
use gity_core::AppError;

pub(super) fn window(params: &ListQuery) -> Result<(u64, u64, usize), AppError> {
  let page = params.page.unwrap_or(1);
  let page_size = params.page_size.unwrap_or(20);
  if page == 0 || !(1..=100).contains(&page_size) {
    return Err(AppError::bad_request("invalid pagination"));
  }
  let offset = (page - 1)
    .checked_mul(page_size)
    .ok_or_else(|| AppError::bad_request("invalid pagination"))?;
  let offset = usize::try_from(offset).map_err(|_| AppError::bad_request("invalid pagination"))?;
  Ok((page, page_size, offset))
}
