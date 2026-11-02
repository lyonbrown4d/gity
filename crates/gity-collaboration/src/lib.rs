//! Typed collaboration application services and SeaORM repository adapters.
mod approval_rules;
mod attachments;
mod catalog;
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
mod application_approval_rules;
mod application_attachments;
mod application_catalog;
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
mod application_wiki_pages;
