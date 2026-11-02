//! Collaboration commands independent of transport and persistence adapters.
use serde::{Deserialize, Deserializer};

/// Missing fields preserve the value; an explicit nullable value clears it.
#[derive(Clone, Debug, Default)]
pub enum Change<T> {
  #[default]
  Unchanged,
  Set(T),
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Change<T> {
  fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
    T::deserialize(d).map(Self::Set)
  }
}
mod issue;
pub use issue::*;
mod merge_request;
pub use merge_request::*;
mod wiki_page;
pub use wiki_page::*;
mod release;
pub use release::*;
mod approval_rule;
pub use approval_rule::*;
mod attachment;
pub use attachment::*;
mod issue_comment;
pub use issue_comment::*;
mod merge_comment;
pub use merge_comment::*;
mod release_link;
pub use release_link::*;
mod catalog;
pub use catalog::*;
mod notification;
pub use notification::Notification;
mod lifecycle;
pub use lifecycle::*;
