//! Shared connection for the typed, resource-specific persistence adapters.
pub(crate) struct CollaborationRepository {
  pub(crate) connection: sea_orm::DatabaseConnection,
}
