use super::*;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseTransaction, FromQueryResult};

pub(super) async fn organization(
    s: &AppState,
    tx: &DatabaseTransaction,
    actor: &gity_core::Principal,
    id: i64,
    minimum: i64,
) -> Result<Value, AppError> {
    let mut row = Value::find_by_statement(s.db.statement(
        "SELECT o.*,(SELECT role FROM organization_members WHERE organization_id=o.id AND user_id=?) AS current_role FROM organizations o WHERE o.id=?",
        vec![actor.user_id.into(),id.into()],
    )).one(tx).await?.ok_or_else(|| AppError::not_found("organization not found"))?;
    if actor.is_super_admin {
        row["current_role"] = json!("owner");
    } else if access::role_level(row["current_role"].as_str().unwrap_or_default()) < minimum {
        return Err(AppError::forbidden("insufficient organization permissions"));
    }
    Ok(row)
}

pub(super) async fn update(s: &AppState, h: &HeaderMap, id: i64, body: &Value) -> Reply {
    let actor = auth::principal(s, h).await?;
    let key = body
        .get("key")
        .map(|key| slug(key.as_str().unwrap_or_default()))
        .transpose()?;
    if body.get("name").is_some() {
        required(body, "name")?;
    }
    if body.get("visibility").is_some() {
        visibility(body)?;
    }
    let tx = s.db.transaction().await?;
    // Namespace edits lock organizations before reading paths. Project creation
    // takes the same row lock, so it cannot commit a stale logical namespace.
    let (lock, args) = if key.is_some() {
        ("UPDATE organizations SET updated_at=updated_at", vec![])
    } else {
        (
            "UPDATE organizations SET updated_at=updated_at WHERE id=?",
            vec![id.into()],
        )
    };
    tx.execute_raw(s.db.statement(lock, args)).await?;
    let current = organization(s, &tx, &actor, id, 50).await?;
    if let Some(key) = &key {
        let old = current["full_path"].as_str().unwrap_or_default();
        let next = current["full_path"]
            .as_str()
            .and_then(|path| path.rsplit_once('/'))
            .map(|(parent, _)| format!("{parent}/{key}"))
            .unwrap_or_else(|| key.clone());
        if old != next {
            rename(s, &tx, old, &next).await?;
        }
    }
    let mut fields = fields(body, &["name", "description", "visibility"]);
    if let Some(key) = key {
        fields.insert("path_key".into(), key.into());
    }
    fields.insert("updated_at".into(), gity_core::now().into());
    let columns = fields
        .keys()
        .map(|column| format!("{}=?", column))
        .collect::<Vec<_>>();
    let mut args = fields
        .into_values()
        .map(gity_core::sql_value)
        .collect::<Result<Vec<_>, _>>()?;
    args.push(id.into());
    tx.execute_raw(s.db.statement(
        &format!("UPDATE organizations SET {} WHERE id=?", columns.join(",")),
        args,
    ))
    .await?;
    let row = organization(s, &tx, &actor, id, 50).await?;
    tx.commit().await?;
    Ok(ok(organization_view(row)))
}

async fn rename(
    s: &AppState,
    tx: &DatabaseTransaction,
    old: &str,
    next: &str,
) -> Result<(), AppError> {
    let suffix = if s.db.backend() == DatabaseBackend::MySql {
        "CONCAT(?,SUBSTR(full_path,?))"
    } else {
        "? || SUBSTR(full_path,?)"
    };
    let escaped = old.replace('!', "!!").replace('%', "!%").replace('_', "!_");
    // A slash boundary prevents a rename of `team` from touching `team-extra`.
    for table in ["organizations", "projects"] {
        tx.execute_raw(s.db.statement(
            &format!("UPDATE {table} SET full_path={suffix},updated_at=? WHERE full_path=? OR full_path LIKE ? ESCAPE '!'"),
            vec![next.into(),((old.len()+1) as i32).into(),gity_core::now().into(),old.into(),format!("{escaped}/%").into()],
        )).await?;
    }
    Ok(())
}
