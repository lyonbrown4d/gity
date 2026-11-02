//! Keep each table, its PostgreSQL comments and its indexes in one ordered block.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

pub(super) async fn create(
  manager: &SchemaManager<'_>,
  table: TableCreateStatement,
  indexes: Vec<(IndexCreateStatement, &'static str)>,
) -> Result<(), DbErr> {
  manager.create_table(table.clone()).await?;
  if manager.get_database_backend() == DbBackend::Postgres {
    table_comments(manager, &table).await?;
  }
  for (index, purpose) in indexes {
    manager.create_index(index.clone()).await?;
    if manager.get_database_backend() == DbBackend::Postgres {
      let name = index
        .get_index_spec()
        .get_name()
        .ok_or_else(|| DbErr::Migration("baseline index is missing a name".into()))?;
      comment(manager, "INDEX", &quote_identifier(name), purpose).await?;
    }
  }
  Ok(())
}

async fn table_comments(
  manager: &SchemaManager<'_>,
  table: &TableCreateStatement,
) -> Result<(), DbErr> {
  let Some(TableRef::Table(TableName(None, name), None)) = table.get_table_name() else {
    return Err(DbErr::Migration(
      "baseline table must have an unqualified name".into(),
    ));
  };
  let table_name = quote_identifier(&name.to_string());
  let description = table
    .get_comment()
    .ok_or_else(|| DbErr::Migration("baseline table is missing a comment".into()))?;
  comment(manager, "TABLE", &table_name, description).await?;
  for column in table.get_columns() {
    let description = column
      .get_column_spec()
      .comment
      .as_ref()
      .ok_or_else(|| DbErr::Migration("baseline column is missing a comment".into()))?;
    let target = format!(
      "{table_name}.{}",
      quote_identifier(&column.get_column_name())
    );
    comment(manager, "COLUMN", &target, description).await?;
  }
  Ok(())
}

async fn comment(
  manager: &SchemaManager<'_>,
  kind: &str,
  target: &str,
  description: &str,
) -> Result<(), DbErr> {
  // SeaQuery 1.0 has no COMMENT ON statement builder. Only schema metadata uses
  // backend-specific SQL; all tables, columns, checks, keys and indexes are typed.
  // Identifiers and literals are escaped even though every input is frozen here.
  let literal = description.replace('\'', "''");
  manager
    .get_connection()
    .execute_unprepared(&format!("COMMENT ON {kind} {target} IS '{literal}'"))
    .await?;
  Ok(())
}

fn quote_identifier(name: &str) -> String {
  format!("\"{}\"", name.replace('"', "\"\""))
}
