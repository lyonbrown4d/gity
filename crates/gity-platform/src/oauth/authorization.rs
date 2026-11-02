use axum::{
  Json,
  extract::{Query, State},
  http::StatusCode,
  response::Redirect,
};
use gity_core::AppError;
use gity_identity::{AuthorizationRequest, ClientRegistration};
use serde::Deserialize;
use serde_json::Value;
use utoipa::ToSchema;

use crate::{AppState, openapi::schemas as api};

#[derive(Deserialize, utoipa::IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub(super) struct AuthorizationQuery {
  #[schema(max_length = 128)]
  client_id: String,
  #[schema(max_length = 2048)]
  redirect_uri: String,
  response_type: String,
  #[schema(min_length = 43, max_length = 43)]
  code_challenge: String,
  code_challenge_method: String,
  #[schema(max_length = 256)]
  scope: String,
  #[schema(min_length = 16, max_length = 512)]
  state: String,
  #[schema(max_length = 2048)]
  resource: String,
  #[serde(default)]
  #[schema(max_length = 512)]
  nonce: Option<String>,
}

impl From<AuthorizationQuery> for AuthorizationRequest {
  fn from(query: AuthorizationQuery) -> Self {
    Self {
      client_id: query.client_id,
      redirect_uri: query.redirect_uri,
      response_type: query.response_type,
      code_challenge: query.code_challenge,
      code_challenge_method: query.code_challenge_method,
      scope: query.scope,
      state: query.state,
      resource: query.resource,
      nonce: query.nonce,
    }
  }
}

#[utoipa::path(
  get,
  path = "/oauth/authorize",
  tag = "oauth",
  description = "Starts an authorization code flow. Public clients must use PKCE S256 and the exact MCP resource URI.",
  params(AuthorizationQuery),
  responses((status = 307, description = "Redirect to authenticated consent page"), api::Errors),
  security(())
)]
pub(super) async fn authorize(
  State(state): State<AppState>,
  Query(query): Query<AuthorizationQuery>,
) -> Result<Redirect, AppError> {
  let location = state
    .oauth
    .begin_authorization(query.into())
    .await
    .map_err(super::map_oauth)?;
  Ok(Redirect::temporary(&location))
}

#[derive(Deserialize, ToSchema)]
pub(super) struct ClientRegistrationBody {
  #[schema(min_length = 1, max_length = 128)]
  client_name: String,
  #[schema(min_items = 1, max_items = 20)]
  redirect_uris: Vec<String>,
}

impl From<ClientRegistrationBody> for ClientRegistration {
  fn from(body: ClientRegistrationBody) -> Self {
    Self {
      client_name: body.client_name,
      redirect_uris: body.redirect_uris,
    }
  }
}

#[utoipa::path(
  post,
  path = "/oauth/register",
  tag = "oauth",
  description = "Registers a bounded public OAuth client with exact HTTPS or loopback callback URIs.",
  request_body = ClientRegistrationBody,
  responses((status = 201, description = "Public client registered", body = Value), api::Errors),
  security(())
)]
pub(super) async fn register(
  State(state): State<AppState>,
  Json(input): Json<ClientRegistrationBody>,
) -> Result<(StatusCode, Json<Value>), AppError> {
  let client = state
    .oauth
    .register_client(input.into())
    .await
    .map_err(super::map_oauth)?;
  Ok((StatusCode::CREATED, Json(client)))
}
