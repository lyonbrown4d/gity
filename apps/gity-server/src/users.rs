mod profile;
use crate::openapi::{schemas as api, user_schemas as spec};
use crate::{AppState, auth};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use gity_core::{AppError, ok};
use profile::update_profile;
use serde_json::{Value, json};
type Reply = Result<Json<Value>, AppError>;
pub fn routes() -> Router<AppState> {
    documented_routes().split_for_parts().0
}
pub fn documented_routes() -> utoipa_axum::router::OpenApiRouter<AppState> {
    utoipa_axum::router::OpenApiRouter::new()
        .routes(utoipa_axum::routes!(list, create))
        .routes(utoipa_axum::routes!(me, update_me))
        .routes(utoipa_axum::routes!(detail, update, remove))
        .routes(utoipa_axum::routes!(tokens, create_token))
}
pub async fn admin(state: &AppState, headers: &HeaderMap) -> Result<(), AppError> {
    if !auth::principal(state, headers).await?.is_super_admin {
        return Err(AppError::forbidden("administrator required"));
    }
    Ok(())
}
#[utoipa::path(
    get,path="/users",tag="users",
    description="Requires an authenticated user.",
    params(spec::UserQuery),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::UserPage>),api::Errors),
    security(("bearer"=[]))
)]
async fn list(
    State(s): State<AppState>,
    h: HeaderMap,
    Query(query): Query<spec::UserQuery>,
) -> Reply {
    auth::principal(&s, &h).await?;
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = page.checked_sub(1).and_then(|p| p.checked_mul(page_size));
    let offset = offset
        .filter(|v| *v <= i64::MAX as u64)
        .filter(|_| (1..=100).contains(&page_size))
        .ok_or_else(|| AppError::bad_request("invalid pagination"))?;
    use sea_orm::ConnectionTrait;
    let total: i64 =
        s.db.connection
            .query_one_raw(s.db.statement("SELECT COUNT(*) AS total FROM users", vec![]))
            .await?
            .ok_or_else(|| AppError::internal("user count unavailable"))?
            .try_get("", "total")?;
    let items =
        s.db.all(
            "SELECT * FROM users ORDER BY id LIMIT ? OFFSET ?",
            vec![(page_size as i64).into(), (offset as i64).into()],
        )
        .await?
        .into_iter()
        .map(view)
        .collect::<Vec<_>>();
    Ok(ok(
        json!({"items":items,"total":total,"page":page,"page_size":page_size}),
    ))
}
#[utoipa::path(
    get,path="/users/me",tag="users",
    description="Returns the authenticated user profile.",
    responses((status=200,description="Operation succeeded",body=api::Envelope<Option<spec::User>>),api::Errors),
    security(("bearer"=[]))
)]
async fn me(State(s): State<AppState>, h: HeaderMap) -> Reply {
    let p = auth::principal(&s, &h).await?;
    Ok(ok(s
        .db
        .one("SELECT * FROM users WHERE id=?", vec![p.user_id.into()])
        .await?
        .map(view)))
}
#[utoipa::path(
    get,path="/users/{id}",tag="users",
    description="Requires authentication.",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::User>),api::Errors),
    security(("bearer"=[]))
)]
async fn detail(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i64>) -> Reply {
    auth::principal(&s, &h).await?;
    Ok(ok(view(
        s.db.one("SELECT * FROM users WHERE id=?", vec![id.into()])
            .await?
            .ok_or_else(|| AppError::not_found("user not found"))?,
    )))
}
#[utoipa::path(
    post,path="/users",tag="users",
    description="Requires a super administrator.",
    request_body=spec::CreateUser,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::User>),api::Errors),
    security(("bearer"=[]))
)]
async fn create(State(s): State<AppState>, h: HeaderMap, Json(b): Json<Value>) -> Reply {
    admin(&s, &h).await?;
    if b["password"]
        .as_str()
        .is_some_and(|password| !(8..=1024).contains(&password.len()))
    {
        return Err(AppError::bad_request("password must contain 8–1024 bytes"));
    }
    let username = b["username"]
        .as_str()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| AppError::bad_request("username required"))?;
    let user=s.db.insert("users",json!({"username":username,"display_name":b["display_name"].as_str().unwrap_or(username),"email":b.get("email"),"is_super_admin":b["is_super_admin"].as_bool().unwrap_or(false)}).as_object().unwrap().clone()).await?;
    if let Some(password) = b["password"].as_str() {
        if password.len() < 8 {
            return Err(AppError::bad_request(
                "password must contain at least 8 bytes",
            ));
        }
        set_password(&s, user["id"].as_i64().unwrap(), password.to_owned()).await?;
    }
    Ok(ok(view(user)))
}
pub async fn set_password(s: &AppState, id: i64, password: String) -> Result<(), AppError> {
    if password.len() < 8 || password.len() > 1024 {
        return Err(AppError::bad_request("password must contain 8–1024 bytes"));
    }
    if s.db
        .one("SELECT id FROM users WHERE id=?", vec![id.into()])
        .await?
        .is_none()
    {
        return Err(AppError::not_found("user not found"));
    }
    let hash = auth::hash_password(password).await?;
    let tx = s.db.transaction().await?;
    use sea_orm::ConnectionTrait;
    tx.execute_raw(s.db.statement(
        "DELETE FROM user_passwords WHERE user_id=?",
        vec![id.into()],
    ))
    .await?;
    tx.execute_raw(s.db.statement(
        "INSERT INTO user_passwords(user_id,password_hash) VALUES (?,?)",
        vec![id.into(), hash.into()],
    ))
    .await?;
    tx.execute_raw(s.db.statement("DELETE FROM auth_sessions WHERE user_id=?", vec![id.into()]))
        .await?;
    tx.commit().await?;
    Ok(())
}
#[utoipa::path(
    patch,path="/users/me",tag="users",
    description="Updates the authenticated profile; cannot change administrator status.",
    request_body=spec::UpdateUser,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::User>),api::Errors),
    security(("bearer"=[]))
)]
async fn update_me(State(s): State<AppState>, h: HeaderMap, Json(b): Json<Value>) -> Reply {
    let p = auth::principal(&s, &h).await?;
    update_profile(&s, p.user_id, &b, false).await
}
#[utoipa::path(
    patch,path="/users/{id}",tag="users",
    description="Only this user or a super administrator can update the profile.",
    params(api::ProjectPath),
    request_body=spec::UpdateUser,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::User>),api::Errors),
    security(("bearer"=[]))
)]
async fn update(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(id): Path<i64>,
    Json(b): Json<Value>,
) -> Reply {
    let p = auth::principal(&s, &h).await?;
    if p.user_id != id && !p.is_super_admin {
        return Err(AppError::forbidden("cannot change another user"));
    }
    update_profile(&s, id, &b, p.is_super_admin).await
}
#[utoipa::path(
    delete,path="/users/{id}",tag="users",
    description="Requires a super administrator; referenced users may not be deleted.",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::OkResult>),api::Errors),
    security(("bearer"=[]))
)]
async fn remove(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i64>) -> Reply {
    admin(&s, &h).await?;
    use sea_orm::ConnectionTrait;
    let tx = s.db.transaction().await?;
    for table in ["auth_sessions", "user_passwords", "user_access_tokens"] {
        tx.execute_raw(s.db.statement(
            &format!("DELETE FROM {table} WHERE user_id=?"),
            vec![id.into()],
        ))
        .await?;
    }
    let deleted = tx
        .execute_raw(s.db.statement("DELETE FROM users WHERE id=?", vec![id.into()]))
        .await?;
    if deleted.rows_affected() == 0 {
        return Err(AppError::not_found("user not found"));
    }
    tx.commit().await?;
    Ok(ok(json!({"ok":true})))
}
async fn own_tokens(s: &AppState, h: &HeaderMap, id: i64) -> Result<(), AppError> {
    let p = auth::principal(s, h).await?;
    if p.user_id != id && !p.is_super_admin {
        return Err(AppError::forbidden(
            "cannot access another user's credentials",
        ));
    }
    Ok(())
}
#[utoipa::path(
    get,path="/users/{id}/tokens",tag="users",
    description="Only this user or an administrator can list token metadata. Raw tokens and hashes are omitted.",
    params(api::ProjectPath),
    responses((status=200,description="Operation succeeded",body=api::Envelope<Vec<spec::UserToken>>),api::Errors),
    security(("bearer"=[]))
)]
async fn tokens(State(s): State<AppState>, h: HeaderMap, Path(id): Path<i64>) -> Reply {
    own_tokens(&s, &h, id).await?;
    Ok(ok(s.db.all("SELECT id,user_id,name,created_at,updated_at FROM user_access_tokens WHERE user_id=? AND name NOT IN ('access','refresh') ORDER BY id",vec![id.into()]).await?))
}
#[utoipa::path(
    post,path="/users/{id}/tokens",tag="users",
    description="Only this user or an administrator; raw token returned once and stored hashed.",
    params(api::ProjectPath),
    request_body=spec::CreateUserToken,
    responses((status=200,description="Operation succeeded",body=api::Envelope<spec::CreatedUserToken>),api::Errors),
    security(("bearer"=[]))
)]
async fn create_token(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(id): Path<i64>,
    Json(b): Json<Value>,
) -> Reply {
    own_tokens(&s, &h, id).await?;
    let name = b["name"]
        .as_str()
        .filter(|v| !v.is_empty() && *v != "access" && *v != "refresh")
        .ok_or_else(|| AppError::bad_request("personal token name required"))?;
    let token = auth::secret();
    let mut row =
        s.db.insert(
            "user_access_tokens",
            json!({"user_id":id,"name":name,"token":format!("sha256:{}",auth::token_hash(&token))})
                .as_object()
                .unwrap()
                .clone(),
        )
        .await?;
    row["token"] = json!(token);
    Ok(ok(row))
}

fn view(mut row: Value) -> Value {
    row["is_super_admin"] = json!(
        row["is_super_admin"]
            .as_bool()
            .unwrap_or_else(|| row["is_super_admin"].as_i64() == Some(1))
    );
    if row["email"].is_null() {
        row["email"] = json!("");
    }
    row
}

#[cfg(test)]
mod profile_tests;

#[cfg(test)]
mod directory_tests;
