//! Backend type differences that SeaQuery cannot express without custom types.

use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

#[derive(DeriveIden)]
enum NativeType {
  #[sea_orm(iden = "BIGINT")]
  SqliteBigInt,
  #[sea_orm(iden = "DATETIME(6)")]
  MySqlDateTime,
  #[sea_orm(iden = "LONGTEXT")]
  MySqlLongText,
}

pub(super) fn big_integer(name: impl IntoIden, backend: DbBackend) -> ColumnDef {
  let mut column = ColumnDef::new(name);
  if backend == DbBackend::Sqlite {
    // SQLite INTEGER PRIMARY KEY aliases ROWID and silently generates omitted IDs.
    // BIGINT retains integer affinity while requiring an explicit NOT NULL key.
    column.custom(NativeType::SqliteBigInt);
  } else {
    column.big_integer();
  }
  column
}

pub(super) fn string(name: impl IntoIden, backend: DbBackend, length: u32) -> ColumnDef {
  let mut column = ColumnDef::new(name);
  if backend == DbBackend::MySql {
    column.string_len(length);
  } else {
    column.text();
  }
  column
}

pub(super) fn text(name: impl IntoIden, backend: DbBackend) -> ColumnDef {
  let mut column = ColumnDef::new(name);
  if backend == DbBackend::MySql {
    // CI scripts, logs, configuration and package metadata must exceed TEXT's 64 KiB limit.
    column.custom(NativeType::MySqlLongText);
  } else {
    column.text();
  }
  column
}

pub(super) fn timestamp(name: impl IntoIden, backend: DbBackend) -> ColumnDef {
  let mut column = ColumnDef::new(name);
  if backend == DbBackend::MySql {
    // DATETIME avoids session time-zone conversion; SeaQuery omits fractional precision.
    column.custom(NativeType::MySqlDateTime);
  } else {
    column.timestamp();
  }
  column
}

pub(super) fn finish_table(table: &mut TableCreateStatement, backend: DbBackend, collation: &str) {
  if backend == DbBackend::MySql {
    table
      .engine("InnoDB")
      .character_set("utf8mb4")
      .collate(collation);
  }
}
