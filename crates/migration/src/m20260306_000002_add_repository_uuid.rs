use sea_orm_migration::prelude::*;
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Repositories::Table)
          .add_column(ColumnDef::new(Repositories::Uuid).string().null())
          .to_owned(),
      )
      .await?;

    let conn = manager.get_connection();
    let select_stmt = Query::select()
      .column(Repositories::Id)
      .from(Repositories::Table)
      .to_owned();
    let rows = conn.query_all(&select_stmt).await?;

    for row in rows {
      let repository_id: String = row.try_get("", Repositories::Id.to_string().as_str())?;
      let update_stmt = Query::update()
        .table(Repositories::Table)
        .value(Repositories::Uuid, Uuid::new_v4().to_string())
        .and_where(Expr::col(Repositories::Id).eq(repository_id))
        .to_owned();
      conn.execute(&update_stmt).await?;
    }

    manager
      .create_index(
        Index::create()
          .name("idx_repositories_uuid")
          .table(Repositories::Table)
          .col(Repositories::Uuid)
          .unique()
          .if_not_exists()
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_index(
        Index::drop()
          .name("idx_repositories_uuid")
          .table(Repositories::Table)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(Repositories::Table)
          .drop_column(Repositories::Uuid)
          .to_owned(),
      )
      .await
  }
}

#[derive(DeriveIden)]
enum Repositories {
  Table,
  Id,
  Uuid,
}
