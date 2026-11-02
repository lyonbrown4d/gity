mod authorization;
mod consent;
mod metadata;
mod tokens;

use axum::{Router, extract::DefaultBodyLimit};
use gity_core::AppError;
use gity_identity::OAuthError;

use crate::AppState;

pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(metadata::authorization_metadata))
    .routes(utoipa_axum::routes!(metadata::resource_metadata))
    .routes(utoipa_axum::routes!(metadata::mcp_resource_metadata))
    .routes(utoipa_axum::routes!(authorization::authorize))
    .routes(utoipa_axum::routes!(authorization::register))
    .routes(utoipa_axum::routes!(tokens::token))
    .routes(utoipa_axum::routes!(tokens::revoke))
    .routes(utoipa_axum::routes!(consent::pending))
    .routes(utoipa_axum::routes!(consent::decide))
    .layer(DefaultBodyLimit::max(64 * 1024))
}

pub fn routes() -> Router<AppState> {
  documented_routes().split_for_parts().0
}

pub(super) fn map_oauth(error: OAuthError) -> AppError {
  match error {
    OAuthError::Invalid => AppError::bad_request("invalid OAuth request"),
    OAuthError::Expired => AppError::unauthorized("OAuth request expired or already used"),
    OAuthError::Database(_) | OAuthError::Configuration => {
      AppError::internal("OAuth service unavailable")
    }
  }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod error_tests;
