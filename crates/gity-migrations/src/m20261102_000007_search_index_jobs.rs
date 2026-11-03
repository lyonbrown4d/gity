//! Durable, coalesced project search rebuild requests.
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum SearchIndexJobs {
  Table,
  Id,
  ProjectId,
  RequestedGeneration,
  CompletedGeneration,
  PublishedToken,
  PublishedGeneration,
  LeaseToken,
  LeaseGeneration,
  LeaseExpiresAt,
  UpdatedAt,
}

#[derive(DeriveIden)]
enum NativeType {
  #[sea_orm(iden = "BIGINT")]
  SqliteBigInt,
  #[sea_orm(iden = "DATETIME(6)")]
  MySqlDateTime,
}

fn integer(name: impl IntoIden, backend: sea_orm::DbBackend) -> ColumnDef {
  let mut column = ColumnDef::new(name);
  if backend == sea_orm::DbBackend::Sqlite {
    column.custom(NativeType::SqliteBigInt);
  } else {
    column.big_integer();
  }
  column
}

fn timestamp(name: impl IntoIden, backend: sea_orm::DbBackend) -> ColumnDef {
  let mut column = ColumnDef::new(name);
  if backend == sea_orm::DbBackend::MySql {
    column.custom(NativeType::MySqlDateTime);
  } else {
    column.timestamp();
  }
  column
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let backend = manager.get_database_backend();
    let mut table = Table::create();
    table
      .table(SearchIndexJobs::Table)
      .comment("按项目合并的持久化搜索投影重建任务")
      .col(
        integer(SearchIndexJobs::Id, backend)
          .comment("应用生成的雪花主键")
          .not_null()
          .primary_key(),
      )
      .col(
        integer(SearchIndexJobs::ProjectId, backend)
          .comment("项目雪花主键，每个项目仅保留一条重建任务")
          .not_null(),
      )
      .col(
        integer(SearchIndexJobs::RequestedGeneration, backend)
          .comment("等待投影的最新请求代数")
          .not_null()
          .default(1),
      )
      .col(
        integer(SearchIndexJobs::CompletedGeneration, backend)
          .comment("已完成投影的请求代数")
          .not_null()
          .default(0),
      )
      .col(
        ColumnDef::new(SearchIndexJobs::PublishedToken)
          .comment("已发布的索引命名空间，包含后端实例标识")
          .string_len(128)
          .null(),
      )
      .col(
        integer(SearchIndexJobs::PublishedGeneration, backend)
          .comment("已发布索引对应的请求代数")
          .not_null()
          .default(0),
      )
      .col(
        ColumnDef::new(SearchIndexJobs::LeaseToken)
          .comment("当前租约的命名空间令牌")
          .string_len(128)
          .null(),
      )
      .col(
        integer(SearchIndexJobs::LeaseGeneration, backend)
          .comment("当前租约处理的请求代数")
          .null(),
      )
      .col(
        timestamp(SearchIndexJobs::LeaseExpiresAt, backend)
          .comment("租约到期时间，使用 UTC")
          .null(),
      )
      .col(
        timestamp(SearchIndexJobs::UpdatedAt, backend)
          .comment("队列调度更新时间，使用 UTC")
          .not_null(),
      );
    manager.create_table(table.to_owned()).await?;
    manager
      .create_index(
        Index::create()
          .name("uq_search_index_jobs_project")
          .table(SearchIndexJobs::Table)
          .col(SearchIndexJobs::ProjectId)
          .unique()
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .name("ix_search_index_jobs_ready")
          .table(SearchIndexJobs::Table)
          .col(SearchIndexJobs::CompletedGeneration)
          .col(SearchIndexJobs::RequestedGeneration)
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(SearchIndexJobs::Table).to_owned())
      .await
  }
}
