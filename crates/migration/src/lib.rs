pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20220102_000001_create_organizations;
mod m20220103_000001_create_repositories_and_invitations;
mod m20220104_000001_create_repo_branches_and_commits;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(m20220101_000001_create_table::Migration),
      Box::new(m20220102_000001_create_organizations::Migration),
      Box::new(m20220103_000001_create_repositories_and_invitations::Migration),
      Box::new(m20220104_000001_create_repo_branches_and_commits::Migration),
    ]
  }
}
