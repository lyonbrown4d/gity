pub use gity_ids::{IdError, IdGenerator, install, new_id, try_new_id};
pub fn now() -> String {
  chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests;
