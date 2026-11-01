pub(super) async fn lock(project: i64) -> tokio::sync::OwnedMutexGuard<()> {
  use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
  };
  static LOCKS: OnceLock<Mutex<HashMap<i64, Arc<tokio::sync::Mutex<()>>>>> = OnceLock::new();
  let lock = {
    let mut locks = LOCKS
      .get_or_init(Default::default)
      .lock()
      .expect("project locks poisoned");
    locks.entry(project).or_default().clone()
  };
  lock.lock_owned().await
}
