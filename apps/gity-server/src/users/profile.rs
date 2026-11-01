use super::*;
use sea_orm::{ConnectionTrait, FromQueryResult};

pub(super) async fn update_profile(s: &AppState, id: i64, body: &Value, admin: bool) -> Reply {
    let input: spec::UpdateUser = serde_json::from_value(body.clone())
        .map_err(|_| AppError::bad_request("invalid profile"))?;
    if !admin && input.status.is_some() {
        return Err(AppError::forbidden("administrator required"));
    }
    let disable = input.status == Some(spec::UserStatus::Disabled);
    let password = input
        .password
        .filter(|password| !password.trim().is_empty());
    if password
        .as_ref()
        .is_some_and(|password| !(8..=1024).contains(&password.len()))
    {
        return Err(AppError::bad_request("password must contain 8–1024 bytes"));
    }
    let mut fields: Vec<(&str, sea_orm::Value)> = Vec::new();
    if body.get("username").is_some() {
        let username = auth::validate_username(input.username.as_deref().unwrap_or_default())?;
        if s.db
            .one(
                "SELECT id FROM users WHERE username=? AND id<>?",
                vec![username.into(), id.into()],
            )
            .await?
            .is_some()
        {
            return Err(AppError::conflict("resource already exists"));
        }
        fields.push(("username", username.to_owned().into()));
    }
    for (key, value) in [("display_name", input.display_name), ("email", input.email)] {
        if body.get(key).is_some() {
            fields.push((key, value.into()));
        }
    }
    if admin && let Some(value) = input.is_super_admin {
        fields.push(("is_super_admin", value.into()));
    }
    if admin && let Some(status) = input.status {
        fields.push(("status", status.as_str().into()));
    }
    // Expensive hashing and all request validation finish before the database
    // transaction. A failed field update cannot leave a changed password behind.
    let hash = if let Some(password) = password {
        Some(auth::hash_password(password).await?)
    } else {
        None
    };
    fields.push(("updated_at", gity_core::now().into()));
    let assignments = fields
        .iter()
        .map(|(column, _)| {
            s.db.quote_identifier(column)
                .map(|column| format!("{column}=?"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut values = fields
        .into_iter()
        .map(|(_, value)| value)
        .collect::<Vec<_>>();
    values.push(id.into());
    let tx = s.db.transaction().await?;
    tx.execute_raw(s.db.statement(
        &format!("UPDATE users SET {} WHERE id=?", assignments.join(",")),
        values,
    ))
    .await?;
    let revoke = hash.is_some() || disable;
    if let Some(hash) = hash {
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
    }
    if revoke {
        tx.execute_raw(
            s.db.statement("DELETE FROM auth_sessions WHERE user_id=?", vec![id.into()]),
        )
        .await?;
    }
    let row =
        Value::find_by_statement(s.db.statement("SELECT * FROM users WHERE id=?", vec![id.into()]))
            .one(&tx)
            .await?
            .ok_or_else(|| AppError::not_found("user not found"))?;
    tx.commit().await?;
    Ok(ok(view(row)))
}
