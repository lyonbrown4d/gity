use super::merge_requests::{git_error, repository};
use super::tags::tag;
use super::*;
use crate::openapi::{release_schemas as spec, schemas as api};
use axum::{
  extract::{Path, Query, State},
  http::HeaderMap,
};
use std::collections::HashMap;

pub(super) fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
  let shared = utoipa_axum::router::OpenApiRouter::new()
    .routes(utoipa_axum::routes!(
      super::tags::tags,
      super::tags::create_tag,
      super::tags::delete_tag
    ))
    .routes(utoipa_axum::routes!(list, create))
    .routes(utoipa_axum::routes!(detail, update, remove))
    .routes(utoipa_axum::routes!(link))
    .routes(utoipa_axum::routes!(unlink));
  utoipa_axum::router::OpenApiRouter::new()
    .nest("/projects", shared.clone())
    .nest("/repos", shared)
}
async fn value(s: &AppState, p: &Value, row: Value) -> Result<Value, AppError> {
  let links = s
    .collaboration
    .release_links(number(p, "id"), number(&row, "id"))
    .await?
    .into_iter()
    .map(model_value)
    .collect::<Vec<_>>();
  let tag = tag(s, p, string(&row, "tag_name")).await?;
  Ok(json!({"release":row,"links":links,"tag":tag}))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get,path="/{id}/releases",tag="releases",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::ReleaseDetail>>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn list(State(s): State<AppState>, h: HeaderMap, Path(p): Path<i64>) -> Reply {
  let project = crate::access::project_action(&s, &h, p, ProjectAction::Read)
    .await?
    .project;
  let mut out = Vec::new();
  for row in s
    .collaboration
    .releases(p)
    .await?
    .into_iter()
    .map(model_value)
  {
    out.push(value(&s, &project, row).await?);
  }
  response(json!(out))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    get,path="/{id}/releases/{release}",tag="releases",
    params(api::ReleasePath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ReleaseDetail>),api::Errors),
    security((), ("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn detail(State(s): State<AppState>, h: HeaderMap, Path((p, r)): Path<(i64, i64)>) -> Reply {
  let project = crate::access::project_action(&s, &h, p, ProjectAction::Read)
    .await?
    .project;
  let row = model_value(&s.collaboration.release(p, r).await?);
  response(value(&s, &project, row).await?)
}
fn time(b: &Value) -> Result<String, AppError> {
  let raw = string(b, "released_at");
  if raw.is_empty() {
    Ok(now())
  } else {
    chrono::DateTime::parse_from_rfc3339(raw)
      .map(|v| v.to_utc().to_rfc3339())
      .map_err(|_| AppError::bad_request("invalid released_at"))
  }
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    post,path="/{id}/releases",tag="releases",
    params(api::ProjectPath),
    request_body(content=spec::CreateRelease),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ReleaseDetail>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn create(
  State(s): State<AppState>,
  h: HeaderMap,
  Path(p): Path<i64>,
  Json(b): Json<Value>,
) -> Reply {
  let authorization =
    crate::access::project_action(&s, &h, p, ProjectAction::RepositoryAdmin).await?;
  let actor = authorization.actor()?;
  let project = authorization.project;
  let _guard = super::merge_requests::lock(p).await;
  let name = required(&b, "tag_name")?;
  let released = time(&b)?;
  if s.collaboration.find_release(p, &name).await?.is_some() {
    return Err(AppError::conflict("release already exists for tag"));
  }
  if tag(&s, &project, &name).await?.is_none() {
    if b["create_tag"] != true {
      return Err(AppError::not_found("release tag not found"));
    }
    let source = if string(&b, "source_ref").is_empty() {
      string(&project, "default_branch")
    } else {
      string(&b, "source_ref")
    };
    s.git
      .create_tag(&repository(&project), &name, source)
      .await
      .map_err(git_error)?;
  }
  let title = if string(&b, "name").trim().is_empty() {
    name.clone()
  } else {
    string(&b, "name").trim().to_owned()
  };
  let row = model_value(
    s.collaboration
      .create_release(ReleaseCreate {
        project_id: p,
        tag_name: name,
        name: title,
        description: string(&b, "description").trim().to_owned(),
        created_by_user_id: actor.user_id,
        released_at: chrono::DateTime::parse_from_rfc3339(&released)
          .map_err(|_| AppError::bad_request("invalid released_at"))?
          .naive_utc(),
      })
      .await?,
  );
  response(value(&s, &project, row).await?)
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    patch,path="/{id}/releases/{release}",tag="releases",
    params(api::ReleasePath),
    request_body(content=spec::UpdateRelease),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ReleaseDetail>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn update(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, r)): Path<(i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  let project = crate::access::project_action(&s, &h, p, ProjectAction::RepositoryAdmin)
    .await?
    .project;
  s.collaboration.release(p, r).await?;
  let mut change = patch(&b, &["name", "description"]);
  if b.get("released_at").is_some() {
    change.insert("released_at".into(), json!(time(&b)?));
  }
  let row = model_value(
    s.collaboration
      .update_release(p, r, command(Value::Object(change))?)
      .await?,
  );
  response(value(&s, &project, row).await?)
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    delete,path="/{id}/releases/{release}",tag="releases",
    params(api::ReleasePath,("delete_tag"=Option<bool>,Query)),
    responses((status=200,description="Operation succeeded",body=api::Envelope<api::Status>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn remove(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, r)): Path<(i64, i64)>,
  Query(q): Query<HashMap<String, String>>,
) -> Reply {
  let project = crate::access::project_action(&s, &h, p, ProjectAction::RepositoryAdmin)
    .await?
    .project;
  let row = model_value(&s.collaboration.release(p, r).await?);
  if q.get("delete_tag").is_some_and(|v| v == "true")
    && tag(&s, &project, string(&row, "tag_name")).await?.is_some()
  {
    s.git
      .delete_tag(&repository(&project), string(&row, "tag_name"))
      .await
      .map_err(git_error)?;
  }
  s.collaboration.delete_release(p, r).await?;
  response(json!({"status":"deleted"}))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    post,path="/{id}/releases/{release}/links",tag="releases",
    params(api::ReleasePath),
    request_body(content=spec::CreateReleaseLink),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::ReleaseLink>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn link(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, r)): Path<(i64, i64)>,
  Json(b): Json<Value>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::RepositoryAdmin).await?;
  s.collaboration.release(p, r).await?;
  let name = required(&b, "name")?;
  let url = required(&b, "url")?;
  let parsed =
    reqwest::Url::parse(&url).map_err(|_| AppError::bad_request("release link url is invalid"))?;
  if !["http", "https"].contains(&parsed.scheme()) || parsed.host_str().is_none() {
    return Err(AppError::bad_request("release link url is invalid"));
  }
  response(model_value(
    s.collaboration
      .create_release_link(
        p,
        r,
        ReleaseLinkCreate {
          project_release_id: r,
          name,
          url,
          link_type: if string(&b, "link_type").is_empty() {
            "other".to_owned()
          } else {
            string(&b, "link_type").to_owned()
          },
        },
      )
      .await?,
  ))
}
#[gity_api_macros::application_operation(prefix = "/api/v1/projects")]
#[utoipa::path(
    delete,path="/{id}/releases/{release}/links/{link}",tag="releases",
    params(api::LinkPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<api::Status>),api::Errors),
    security(("bearer"=[]), ("basic"=[]), ("private_token"=[]))
)]
async fn unlink(
  State(s): State<AppState>,
  h: HeaderMap,
  Path((p, r, l)): Path<(i64, i64, i64)>,
) -> Reply {
  crate::access::project_action(&s, &h, p, ProjectAction::RepositoryAdmin).await?;
  s.collaboration.release(p, r).await?;
  s.collaboration.delete_release_link(p, r, l).await?;
  response(json!({"status":"deleted"}))
}
