pub(crate) use gity_ids::configure_node;
pub use gity_ids::new_id;
pub fn now() -> String {
  chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests;
