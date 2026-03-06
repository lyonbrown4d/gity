use crate::BaseRepository;
use entity::repository_commits;
use sea_orm::{
  ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
  QueryOrder, QuerySelect,
};

#[derive(Clone)]
pub struct RepositoryCommitsRepository<C: ConnectionTrait = DatabaseConnection> {
  base: BaseRepository<C>,
}

impl<C: ConnectionTrait> RepositoryCommitsRepository<C> {
  pub fn new(conn: C) -> Self {
    Self {
      base: BaseRepository::new(conn),
    }
  }

  pub fn connection(&self) -> &C {
    self.base.connection()
  }

  pub async fn list_commits_by_repo(
    &self,
    repository_id: &str,
    branch_name: Option<String>,
    limit: u64,
  ) -> Result<Vec<repository_commits::Model>, DbErr> {
    let mut finder = repository_commits::Entity::find()
      .filter(repository_commits::Column::RepositoryId.eq(repository_id.to_string()));
    if let Some(branch_name) = branch_name {
      finder = finder.filter(repository_commits::Column::BranchName.eq(branch_name));
    }

    finder
      .order_by_desc(repository_commits::Column::CreatedAt)
      .limit(limit)
      .all(self.connection())
      .await
  }

  pub async fn exists_commit_by_repo_and_sha(
    &self,
    repository_id: &str,
    commit_sha: &str,
  ) -> Result<bool, DbErr> {
    Ok(
      repository_commits::Entity::find()
        .filter(
          Condition::all()
            .add(repository_commits::Column::RepositoryId.eq(repository_id.to_string()))
            .add(repository_commits::Column::CommitSha.eq(commit_sha.to_string())),
        )
        .one(self.connection())
        .await?
        .is_some(),
    )
  }

  pub async fn insert_commit(
    &self,
    active: repository_commits::ActiveModel,
  ) -> Result<repository_commits::Model, DbErr> {
    self.base.insert(active).await
  }
}
