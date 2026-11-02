use super::*;
mod audit;

impl Db {
  pub fn quote_identifier(&self, value: &str) -> Result<String, AppError> {
    identifier(value)?;
    Ok(if self.backend() == DatabaseBackend::MySql {
      format!("`{value}`")
    } else {
      format!("\"{value}\"")
    })
  }
  pub fn statement(&self, sql: &str, args: Vec<Value>) -> Statement {
    let args = if matches!(
      self.backend(),
      DatabaseBackend::Postgres | DatabaseBackend::MySql
    ) {
      bindings::postgres_values(sql, args)
    } else {
      args
    };
    // Application SQL uses ? placeholders; PostgreSQL requires numbered parameters.
    let sql = if self.backend() == DatabaseBackend::Postgres {
      let mut n = 0;
      let mut quoted = false;
      sql
        .chars()
        .map(|c| {
          if c == '\'' {
            quoted = !quoted;
          }
          if c == '?' && !quoted {
            n += 1;
            format!("${n}")
          } else {
            c.to_string()
          }
        })
        .collect::<String>()
    } else {
      sql.to_owned()
    };
    Statement::from_sql_and_values(self.backend(), sql, args)
  }
  pub async fn transaction(&self) -> Result<DatabaseTransaction, AppError> {
    Ok(self.connection.begin().await?)
  }
  pub async fn all(&self, sql: &str, args: Vec<Value>) -> Result<Vec<Json>, AppError> {
    let mut rows = Json::find_by_statement(self.statement(sql, args))
      .all(&self.connection)
      .await?;
    for row in &mut rows {
      crate::error::normalize_timestamps(row);
    }
    Ok(rows)
  }
  pub async fn one(&self, sql: &str, args: Vec<Value>) -> Result<Option<Json>, AppError> {
    let mut row = Json::find_by_statement(self.statement(sql, args))
      .one(&self.connection)
      .await?;
    if let Some(row) = &mut row {
      crate::error::normalize_timestamps(row);
    }
    Ok(row)
  }
  pub async fn execute(&self, sql: &str, args: Vec<Value>) -> Result<u64, AppError> {
    Ok(
      self
        .connection
        .execute_raw(self.statement(sql, args))
        .await?
        .rows_affected(),
    )
  }
  pub async fn insert(&self, table: &str, mut fields: Map<String, Json>) -> Result<Json, AppError> {
    let raw_table = table.to_owned();
    let table = self.quote_identifier(table)?;
    fields.entry("id").or_insert(Json::from(new_id()));
    fields.entry("created_at").or_insert(Json::from(now()));
    fields.entry("updated_at").or_insert(Json::from(now()));
    let id = fields["id"]
      .as_i64()
      .ok_or_else(|| AppError::bad_request("invalid id"))?;
    let mut columns = Vec::new();
    let mut values = Vec::new();
    for (key, value) in fields {
      columns.push(self.quote_identifier(&key)?);
      values.push(sql_value(value)?);
    }
    let sql = format!(
      "INSERT INTO {table} ({}) VALUES ({})",
      columns.join(","),
      vec!["?"; columns.len()].join(",")
    );
    let tx = self.transaction().await?;
    tx.execute_raw(self.statement(&sql, values)).await?;
    let row = Json::find_by_statement(self.statement(
      &format!("SELECT * FROM {table} WHERE id=?"),
      vec![id.into()],
    ))
    .one(&tx)
    .await?
    .ok_or_else(|| AppError::not_found("resource not found"))?;
    self
      .audit_snapshot(&tx, &raw_table, &row, "insert", None)
      .await?;
    tx.commit().await?;
    Ok(row)
  }
  pub async fn update(
    &self,
    table: &str,
    id: i64,
    mut fields: Map<String, Json>,
  ) -> Result<Json, AppError> {
    let raw_table = table.to_owned();
    let table = self.quote_identifier(table)?;
    fields.remove("id");
    fields.remove("created_at");
    fields.insert("updated_at".into(), now().into());
    let mut columns = Vec::new();
    let mut values = Vec::new();
    for (key, value) in fields {
      columns.push(format!("{} = ?", self.quote_identifier(&key)?));
      values.push(sql_value(value)?);
    }
    values.push(id.into());
    let tx = self.transaction().await?;
    tx.execute_raw(self.statement(
      &format!("UPDATE {table} SET {} WHERE id=?", columns.join(",")),
      values,
    ))
    .await?;
    let row = Json::find_by_statement(self.statement(
      &format!("SELECT * FROM {table} WHERE id=?"),
      vec![id.into()],
    ))
    .one(&tx)
    .await?
    .ok_or_else(|| AppError::not_found("resource not found"))?;
    self
      .audit_snapshot(&tx, &raw_table, &row, "update", None)
      .await?;
    tx.commit().await?;
    Ok(row)
  }
  pub async fn delete(&self, table: &str, id: i64) -> Result<(), AppError> {
    let raw_table = table.to_owned();
    let table = self.quote_identifier(table)?;
    let tx = self.transaction().await?;
    if let Some(row) = Json::find_by_statement(self.statement(
      &format!("SELECT * FROM {table} WHERE id=?"),
      vec![id.into()],
    ))
    .one(&tx)
    .await?
    {
      tx.execute_raw(self.statement(&format!("DELETE FROM {table} WHERE id=?"), vec![id.into()]))
        .await?;
      self
        .audit_snapshot(&tx, &raw_table, &row, "delete", None)
        .await?;
    }
    tx.commit().await?;
    Ok(())
  }
}

fn identifier(value: &str) -> Result<(), AppError> {
  if value.is_empty()
    || !value
      .bytes()
      .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
  {
    return Err(AppError::bad_request("invalid field"));
  }
  Ok(())
}
pub fn sql_value(value: Json) -> Result<Value, AppError> {
  Ok(match value {
    Json::Null => Value::String(None),
    Json::Bool(v) => v.into(),
    Json::Number(v) => v
      .as_i64()
      .ok_or_else(|| AppError::bad_request("integer required"))?
      .into(),
    Json::String(v) => v.into(),
    _ => return Err(AppError::bad_request("scalar field required")),
  })
}
