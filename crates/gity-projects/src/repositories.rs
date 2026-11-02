use sea_orm::DatabaseConnection;

/// Typed transaction adapter for project aggregates and membership invariants.
#[derive(Clone)]
pub struct ProjectRepository {
  pub(crate) connection: DatabaseConnection,
}
impl ProjectRepository {
  pub(crate) fn new(connection: DatabaseConnection) -> Self {
    Self { connection }
  }
}
/// Typed persistence adapter for project credentials, variables, protections and LFS.
#[derive(Clone)]
pub struct RepositoryAccessRepository {
  pub(crate) connection: DatabaseConnection,
}
impl RepositoryAccessRepository {
  pub(crate) fn new(connection: DatabaseConnection) -> Self {
    Self { connection }
  }
}
