//! CI application services and typed persistence adapters.
mod application;
mod domain;
mod jobs;
mod module;
mod pipelines;
mod repository;
mod runners;
mod trace;
mod worker;
pub use domain::*;
pub use module::CiModule;
use sea_orm::DatabaseConnection;
#[derive(Clone)]
pub struct CiService {
  repository: repository::CiRepository,
}
impl CiService {
  pub fn new(connection: DatabaseConnection) -> Self {
    Self {
      repository: repository::CiRepository { connection },
    }
  }
}
