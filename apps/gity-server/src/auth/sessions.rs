use super::*;

pub(super) async fn create_account(
    state: &AppState,
    body: Credentials,
) -> Result<Json<Value>, AppError> {
    let hash = hash_password(body.password).await?;
    let id = new_id();
    let org_id = new_id();
    let username = body.username.trim().to_owned();
    let timestamp = now();
    let tx = state.db.transaction().await?;
    let first = tx
        .query_one_raw(state.db.statement("SELECT id FROM users LIMIT 1", vec![]))
        .await?
        .is_none();
    if first {
        tx.execute_raw(state.db.statement(
            "INSERT INTO bootstrap_claim(id) VALUES (?)",
            vec![1i64.into()],
        ))
        .await?;
    }
    tx.execute_raw(state.db.statement("INSERT INTO users(id,username,display_name,email,is_super_admin,created_at,updated_at) VALUES (?,?,?,?,?,?,?)",vec![id.into(),username.clone().into(),username.clone().into(),format!("{username}@local.gity").into(),first.into(),timestamp.clone().into(),timestamp.clone().into()])).await?;
    tx.execute_raw(state.db.statement(
        "INSERT INTO user_passwords(user_id,password_hash) VALUES (?,?)",
        vec![id.into(), hash.into()],
    ))
    .await?;
    let path = format!("{}-{}", username.to_lowercase(), id);
    tx.execute_raw(state.db.statement("INSERT INTO organizations(id,name,path_key,full_path,description,visibility,created_at,updated_at) VALUES (?,?,?,?,?,?,?,?)",vec![org_id.into(),username.clone().into(),path.clone().into(),path.into(),"Personal namespace".into(),"private".into(),timestamp.clone().into(),timestamp.clone().into()])).await?;
    tx.execute_raw(state.db.statement("INSERT INTO organization_members(id,organization_id,user_id,role,created_at,updated_at) VALUES (?,?,?,?,?,?)",vec![new_id().into(),org_id.into(),id.into(),"owner".into(),timestamp.clone().into(),timestamp.into()])).await?;
    tx.commit().await?;
    session(
        state,
        &json!({"id":id,"username":username,"organization_id":org_id,"status":"active"}),
        None,
    )
    .await
}
pub(super) async fn session(
    state: &AppState,
    user: &Value,
    family: Option<String>,
) -> Result<Json<Value>, AppError> {
    from_user(user)?;
    let id = user["id"]
        .as_i64()
        .ok_or_else(|| AppError::internal("invalid user"))?;
    let access = secret();
    let refresh = secret();
    let family = family.unwrap_or_else(secret);
    let tx = state.db.transaction().await?;
    for (token, kind, hours) in [(&access, "access", 1), (&refresh, "refresh", 24 * 30)] {
        let expires = (chrono::Utc::now() + chrono::Duration::hours(hours))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        tx.execute_raw(state.db.statement("INSERT INTO auth_sessions(token_hash,user_id,kind,expires_at,family) VALUES (?,?,?,?,?)",vec![token_hash(token).into(),id.into(),kind.into(),expires.into(),family.clone().into()])).await?;
    }
    tx.commit().await?;
    Ok(ok(
        json!({"user_id":id.to_string(),"username":user["username"],"organization_id":user.get("organization_id"),"token":access,"refresh_token":refresh}),
    ))
}
