use leptos::prelude::*;

/// Counts in-flight API requests so the shell can show route-level feedback.
///
/// Queries register themselves here, so a progress bar or a page skeleton keeps
/// working for every page without each page wiring its own indicator.
#[derive(Clone, Copy, Debug)]
pub struct Pending {
  count: RwSignal<usize>,
}

impl Default for Pending {
  fn default() -> Self {
    Self {
      count: RwSignal::new(0),
    }
  }
}

impl Pending {
  /// Provides the counter for this application; call once at the shell root.
  pub fn provide() -> Self {
    let pending = Self::default();
    provide_context(pending);
    pending
  }
  /// Resolves the counter from context, falling back to a detached one.
  pub fn from_context() -> Self {
    use_context::<Self>().unwrap_or_default()
  }
  pub fn start(&self) {
    self.count.update(|count| *count += 1);
  }
  pub fn finish(&self) {
    self.count.update(|count| *count = count.saturating_sub(1));
  }
  pub fn count(&self) -> RwSignal<usize> {
    self.count
  }
}
