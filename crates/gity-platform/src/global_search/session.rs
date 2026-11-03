use super::SearchKind;
use gity_search::global::SearchDocument;
use std::time::Instant;
#[derive(Clone)]
pub struct Group {
  pub kind: SearchKind,
  pub documents: Vec<SearchDocument>,
  pub offset: usize,
  pub returned_files: std::collections::HashSet<String>,
}
#[derive(Clone)]
pub struct Session {
  pub user: i64,
  pub scope: String,
  pub request: String,
  pub expires: Instant,
  pub groups: Vec<Group>,
  pub partial: bool,
}

impl Session {
  /// Bound retained candidate strings as well as the number of cached cursors.
  pub fn estimated_bytes(&self) -> usize {
    std::mem::size_of::<Self>()
      + self.scope.capacity()
      + self.request.capacity()
      + self
        .groups
        .iter()
        .map(|group| {
          group.documents.capacity() * std::mem::size_of::<SearchDocument>()
            + group
              .documents
              .iter()
              .map(|d| {
                d.key.capacity()
                  + d.resource_id.capacity()
                  + d.version.capacity()
                  + d.title.capacity()
                  + d.body.capacity()
                  + d.path.capacity()
              })
              .sum::<usize>()
            + group.returned_files.capacity() * std::mem::size_of::<String>()
            + group
              .returned_files
              .iter()
              .map(String::capacity)
              .sum::<usize>()
        })
        .sum::<usize>()
  }
}
