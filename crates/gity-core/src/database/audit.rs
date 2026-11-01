use super::*;
use std::future::Future;
tokio::task_local! {static ACTOR:Option<i64>;}

pub async fn context<F: Future>(actor: Option<i64>, future: F) -> F::Output {
  ACTOR.scope(actor, future).await
}

impl Db {
  /// Persist revision and non-secret row snapshot in the caller's transaction.
  pub async fn audit_snapshot(
    &self,
    tx: &DatabaseTransaction,
    table: &str,
    row: &Json,
    operation: &str,
    actor: Option<i64>,
  ) -> Result<(), AppError> {
    let (audit_table, entity_id, fields): (&str, &str, &[&str]) = match table {
      "project_branch_protections" => (
        "project_branch_protection_audits",
        "project_branch_protection_id",
        &[
          "project_id",
          "branch_name",
          "rule_type",
          "push_access_level",
          "merge_access_level",
          "require_merge_request",
          "require_pipeline_success",
          "allow_force_push",
          "allow_delete",
          "created_at",
          "updated_at",
        ],
      ),
      "project_members" => (
        "project_member_audits",
        "project_member_id",
        &["project_id", "user_id", "role", "created_at", "updated_at"],
      ),
      "project_merge_request_approval_rules" => (
        "project_merge_request_approval_rule_audits",
        "project_merge_request_approval_rule_id",
        &[
          "project_id",
          "name",
          "target_branch",
          "approvals_required",
          "eligible_user_ids",
          "code_owner",
          "created_at",
          "updated_at",
        ],
      ),
      "project_ci_variables" => (
        "project_ci_variable_audits",
        "project_ci_variable_id",
        &[
          "project_id",
          "key",
          "masked",
          "protected",
          "created_at",
          "updated_at",
        ],
      ),
      "project_deploy_keys" => (
        "project_deploy_key_audits",
        "project_deploy_key_id",
        &[
          "project_id",
          "title",
          "fingerprint",
          "can_push",
          "created_by_user_id",
          "last_used_at",
          "created_at",
          "updated_at",
        ],
      ),
      _ => return Ok(()),
    };
    let revision = new_id();
    let actor = actor.or_else(|| ACTOR.try_with(|v| *v).ok().flatten());
    tx.execute_raw(self.statement("INSERT INTO audit_revisions(id,created_at,actor,tenant,reason,metadata) VALUES (?,?,?,?,?,?)",vec![revision.into(),now().into(),actor.map(|v|v.to_string()).into(),row["project_id"].as_i64().map(|v|v.to_string()).into(),operation.into(),"{}".into()])).await?;
    let mut values = vec![
      Value::from(revision),
      Value::from(operation),
      sql_value(row["id"].clone())?,
    ];
    let mut columns = vec![
      self.quote_identifier("revision_id")?,
      self.quote_identifier("operation")?,
      self.quote_identifier(entity_id)?,
    ];
    for field in fields {
      columns.push(self.quote_identifier(field)?);
      values.push(sql_value(row[*field].clone())?);
    }
    tx.execute_raw(self.statement(
      &format!(
        "INSERT INTO {audit_table} ({}) VALUES ({})",
        columns.join(","),
        vec!["?"; values.len()].join(",")
      ),
      values,
    ))
    .await?;
    Ok(())
  }
}
