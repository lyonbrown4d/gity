use super::*;
use sea_orm::{ConnectionTrait, FromQueryResult};

pub(super) enum Change<'a> {
    Add(&'a str),
    Set(&'a str),
    Remove,
}

pub(super) async fn mutate(
    s: &AppState,
    headers: &HeaderMap,
    organization: i64,
    user: i64,
    change: Change<'_>,
) -> Reply {
    let actor = auth::principal(s, headers).await?;
    let tx = s.db.transaction().await?;
    // Lock the organization before inspecting any membership. The no-change
    // UPDATE also obtains SQLite's write lock without a deferred-read upgrade.
    tx.execute_raw(s.db.statement(
        "UPDATE organizations SET updated_at=updated_at WHERE id=?",
        vec![organization.into()],
    ))
    .await?;
    Value::find_by_statement(s.db.statement(
        "SELECT id FROM organizations WHERE id=?",
        vec![organization.into()],
    ))
    .one(&tx)
    .await?
    .ok_or_else(|| AppError::not_found("organization not found"))?;
    if !actor.is_super_admin {
        let membership = Value::find_by_statement(s.db.statement(
            "SELECT role FROM organization_members WHERE organization_id=? AND user_id=?",
            vec![organization.into(), actor.user_id.into()],
        ))
        .one(&tx)
        .await?;
        if membership.is_none_or(|row| row["role"] != "owner") {
            return Err(AppError::forbidden("insufficient organization permissions"));
        }
    }
    let existing = Value::find_by_statement(s.db.statement(
        "SELECT id,role FROM organization_members WHERE organization_id=? AND user_id=?",
        vec![organization.into(), user.into()],
    ))
    .one(&tx)
    .await?;
    if matches!(change, Change::Add(_)) && existing.is_some() {
        return Err(AppError::conflict("organization member already exists"));
    }
    if !matches!(change, Change::Add(_)) && existing.is_none() {
        return Err(AppError::not_found("organization member not found"));
    }
    let keeps_owner = matches!(change, Change::Add("owner") | Change::Set("owner"));
    if !keeps_owner && existing.as_ref().is_some_and(|row| row["role"] == "owner") {
        let count = tx.query_one_raw(s.db.statement("SELECT COUNT(*) AS total FROM organization_members WHERE organization_id=? AND role='owner'", vec![organization.into()])).await?
            .ok_or_else(|| AppError::internal("database operation failed"))?.try_get::<i64>("", "total")?;
        if count <= 1 {
            return Err(AppError::conflict(
                "organization must retain at least one owner",
            ));
        }
    }
    match change {
        Change::Add(role) => {
            Value::find_by_statement(
                s.db.statement("SELECT id FROM users WHERE id=?", vec![user.into()]),
            )
            .one(&tx)
            .await?
            .ok_or_else(|| AppError::not_found("user not found"))?;
            tx.execute_raw(s.db.statement("INSERT INTO organization_members(id,organization_id,user_id,role,created_at,updated_at) VALUES (?,?,?,?,?,?)",
                vec![gity_core::new_id().into(),organization.into(),user.into(),role.into(),gity_core::now().into(),gity_core::now().into()])).await?;
        }
        Change::Set(role) => {
            tx.execute_raw(s.db.statement("UPDATE organization_members SET role=?,updated_at=? WHERE organization_id=? AND user_id=?", vec![role.into(),gity_core::now().into(),organization.into(),user.into()])).await?;
        }
        Change::Remove => {
            tx.execute_raw(s.db.statement(
                "DELETE FROM organization_members WHERE organization_id=? AND user_id=?",
                vec![organization.into(), user.into()],
            ))
            .await?;
            tx.commit().await?;
            return Ok(ok(json!({"ok":true})));
        }
    }
    let row = Value::find_by_statement(s.db.statement("SELECT m.*,u.username,u.display_name,u.email FROM organization_members m JOIN users u ON u.id=m.user_id WHERE m.organization_id=? AND m.user_id=?", vec![organization.into(),user.into()])).one(&tx).await?
        .ok_or_else(|| AppError::not_found("organization member not found"))?;
    tx.commit().await?;
    Ok(ok(row))
}
