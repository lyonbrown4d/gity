use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
#[derive(Clone, Debug)]
pub struct ProjectToken {
    pub id: i64,
    pub project_id: i64,
    pub created_by_user_id: i64,
    pub username: String,
    pub scopes: Vec<String>,
    pub kind: String,
}
impl ProjectToken {
    pub fn allows(&self, scope: &str) -> bool {
        self.scopes.iter().any(|v| v == scope)
    }
}
pub fn token_from_headers(h: &HeaderMap) -> Option<String> {
    let value = h
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if value
        .get(..7)
        .is_some_and(|v| v.eq_ignore_ascii_case("bearer "))
    {
        return Some(value[7..].trim().into());
    }
    if value
        .get(..6)
        .is_some_and(|v| v.eq_ignore_ascii_case("basic "))
    {
        let decoded = String::from_utf8(STANDARD.decode(value[6..].trim()).ok()?).ok()?;
        return decoded.split_once(':').map(|(_, p)| p.to_owned());
    }
    h.get("private-token")
        .or_else(|| h.get("deploy-token"))
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}
pub(super) fn active(v: &Value) -> bool {
    if !v["revoked_at"].is_null() {
        return false;
    }
    let expires = text(v, "expires_at");
    v["expires_at"].is_null() || timestamp(expires).is_some_and(|v| v > chrono::Utc::now())
}
pub async fn authenticate_token(s: &AppState, raw: &str) -> Result<Option<ProjectToken>, AppError> {
    let digest = format!("sha256:{}", crate::auth::token_hash(raw));
    let Some(row) =
        s.db.one(
            "SELECT * FROM project_access_tokens WHERE token = ?",
            vec![digest.into()],
        )
        .await?
    else {
        return Ok(None);
    };
    if !active(&row) {
        return Err(AppError::unauthorized("project token is inactive"));
    }
    s.db.update(
        "project_access_tokens",
        num(&row, "id"),
        map(json!({"last_used_at":now()})),
    )
    .await?;
    Ok(Some(ProjectToken {
        id: num(&row, "id"),
        project_id: num(&row, "project_id"),
        created_by_user_id: num(&row, "created_by_user_id"),
        username: text(&row, "username").into(),
        scopes: text(&row, "scopes")
            .split(',')
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_owned)
            .collect(),
        kind: text(&row, "kind").into(),
    }))
}
#[cfg(test)]
mod tests;
