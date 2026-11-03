//! Typed collaboration application services and SeaORM repository adapters.
pub mod activity;
mod approval_rules;
mod attachments;
mod catalog;
mod discussions;
pub mod domain;
mod issue_comments;
mod issues;
mod merge_comments;
mod merge_requests;
mod module;
mod notifications;
mod participants;
mod queries;
mod release_links;
mod releases;
mod repository;
mod search_visibility;
mod subscriptions;
mod wiki_pages;

pub use module::CollaborationModule;

pub struct CollaborationService {
  repository: repository::CollaborationRepository,
}
impl CollaborationService {
  pub fn new(connection: sea_orm::DatabaseConnection) -> Self {
    Self {
      repository: repository::CollaborationRepository { connection },
    }
  }
}
mod milestones;
pub use domain::Notification;
mod application_activity;
mod application_approval_rules;
mod application_attachments;
mod application_catalog;
mod application_discussions;
mod application_issue_comments;
mod application_issues;
mod application_merge_comments;
mod application_merge_requests;
mod application_milestones;
mod application_notifications;
mod application_participants;
mod application_queries;
mod application_release_links;
mod application_releases;
mod application_subscriptions;
mod application_wiki_pages;

mod merge_operation_claim;
mod merge_operation_events;
mod merge_operation_finish;
mod merge_operation_guard;
pub use merge_operation_claim::MergeOperationClaim;
pub use merge_operation_guard::MergeOperationGuard;

pub use merge_operation_finish::MergeRejection;
