use super::*;

/// Identity for a single authorized project operation. A token creator's role
/// and administrator privileges are deliberately absent from this context.
#[derive(Clone, Debug)]
pub struct ProjectActor {
    pub user_id: i64,
    pub username: String,
    pub project_id: i64,
    pub token_id: Option<i64>,
}
pub struct ProjectAuthorization {
    pub project: Value,
    actor: Option<ProjectActor>,
    token: Option<crate::registry::ProjectToken>,
    level: i64,
}
impl ProjectAuthorization {
    pub fn role_level(&self) -> i64 {
        self.level
    }
    pub fn is_project_token(&self) -> bool {
        self.token.is_some()
    }
    pub fn allows(&self, action: ProjectAction) -> bool {
        self.token.as_ref().map_or_else(
            || self.level >= action.minimum_role(),
            |token| {
                action
                    .token_scopes()
                    .iter()
                    .any(|scope| token.allows(scope))
            },
        )
    }
    pub fn actor(&self) -> Result<ProjectActor, AppError> {
        self.actor
            .clone()
            .ok_or_else(|| AppError::unauthorized("authentication required"))
    }
}
pub async fn project_action(
    state: &AppState,
    headers: &HeaderMap,
    id: i64,
    action: ProjectAction,
) -> Result<ProjectAuthorization, AppError> {
    let raw = crate::registry::token_from_headers(headers);
    if let Some(raw) = &raw
        && let Some(token) = crate::registry::authenticate_token(state, raw).await?
    {
        if token.project_id != id
            || !action
                .token_scopes()
                .iter()
                .any(|scope| token.allows(scope))
        {
            return Err(AppError::forbidden(
                "project token scope does not permit this operation",
            ));
        }
        let project = state
            .db
            .one(
                "SELECT * FROM projects WHERE id=? AND deleted_at IS NULL",
                vec![id.into()],
            )
            .await?
            .ok_or_else(|| AppError::not_found("project not found"))?;
        return Ok(ProjectAuthorization {
            project,
            actor: Some(ProjectActor {
                user_id: token.created_by_user_id,
                username: token.username.clone(),
                project_id: id,
                token_id: Some(token.id),
            }),
            token: Some(token),
            level: 0,
        });
    }
    let mut headers = headers.clone();
    if let Some(raw) = raw {
        headers.insert(
            "authorization",
            format!("Bearer {raw}")
                .parse()
                .map_err(|_| AppError::unauthorized("invalid token"))?,
        );
    }
    let project = project(state, &headers, id, action.minimum_role()).await?;
    let mut level = 0;
    let actor = if headers.contains_key("authorization") {
        let user = auth::principal(state, &headers).await?;
        level = super::level(state, &user, &project).await?;
        Some(ProjectActor {
            user_id: user.user_id,
            username: user.username,
            project_id: id,
            token_id: None,
        })
    } else {
        None
    };
    Ok(ProjectAuthorization {
        project,
        actor,
        token: None,
        level,
    })
}
