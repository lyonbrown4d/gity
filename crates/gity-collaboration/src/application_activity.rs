use crate::{CollaborationService, activity::ActivityItem};
use gity_core::AppError;

impl CollaborationService {
  /// Recent audit events from every project the user can read, newest first.
  ///
  /// The limit is bounded here as well as at the HTTP boundary so protocol
  /// adapters cannot request an unbounded scan of the audit table.
  pub async fn activity(&self, user: i64, limit: u64) -> Result<Vec<ActivityItem>, AppError> {
    self.repository.activity(user, limit.clamp(1, 100)).await
  }
}
