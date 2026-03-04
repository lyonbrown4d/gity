use entity::repository_commits;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, QueryFilter,
  QueryOrder, QuerySelect,
};

pub struct RepositoryCommitsRepository;

impl RepositoryCommitsRepository {
  pub async fn list_commits_by_repo<C: ConnectionTrait>(
    conn: &C,
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
      .all(conn)
      .await
  }

  pub async fn exists_commit_by_repo_and_sha<C: ConnectionTrait>(
    conn: &C,
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
        .one(conn)
        .await?
        .is_some(),
    )
  }

  pub async fn insert_commit<C: ConnectionTrait>(
    conn: &C,
    active: repository_commits::ActiveModel,
  ) -> Result<repository_commits::Model, DbErr> {
    active.insert(conn).await
  }
}
