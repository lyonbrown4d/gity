use super::{AngosConfig, command, probe};
use std::{
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
  time::Duration,
};
use tokio::{process::Child, time::Instant};
pub(super) async fn supervise(
  config: AngosConfig,
  mut child: Child,
  stop: tokio_util::sync::CancellationToken,
  healthy: Arc<AtomicBool>,
) {
  let mut restarts = 0;
  loop {
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    let mut failures = 0_u8;
    let exited = loop {
      tokio::select! {
          _ = stop.cancelled() => {
              stop_child(&mut child, config.shutdown_timeout).await;
              healthy.store(false, Ordering::Release);
              return;
          }
          status = child.wait() => {
              healthy.store(false, Ordering::Release);
              match status { Ok(status) => { tracing::warn!(%status, "Angos exited"); }, Err(error) => tracing::error!(%error, "Angos wait failed") }
              break true;
          }
          _ = interval.tick() => {
              if probe(&config.health_url).await { failures = 0; }
              else {
                  failures = failures.saturating_add(1);
                  tracing::warn!(failures, "Angos readiness probe failed");
                  if failures >= 3 { let _ = child.start_kill(); break true; }
              }
          }
      }
    };
    if !exited || stop.is_cancelled() {
      return;
    }
    let _ = child.wait().await;
    if restarts >= config.restart_limit {
      healthy.store(false, Ordering::Release);
      tracing::error!("Angos exceeded restart limit");
      return;
    }
    restarts += 1;
    tokio::select! {
        _ = stop.cancelled() => { healthy.store(false, Ordering::Release); return; },
        _ = tokio::time::sleep(Duration::from_millis(500 * 2_u64.pow(restarts.min(6)))) => {}
    }
    let mut command = command(&config);
    match command.spawn() {
      Ok(mut next) => {
        let deadline = Instant::now() + config.startup_timeout;
        loop {
          if next.try_wait().ok().flatten().is_some() {
            tracing::error!("Angos restart exited before healthy");
            return;
          }
          if probe(&config.health_url).await {
            child = next;
            healthy.store(true, Ordering::Release);
            break;
          }
          if Instant::now() >= deadline {
            let _ = next.kill().await;
            tracing::error!("Angos restart readiness timed out");
            return;
          }
          tokio::select! { _ = stop.cancelled() => { stop_child(&mut next, config.shutdown_timeout).await; healthy.store(false, Ordering::Release); return; }, _ = tokio::time::sleep(Duration::from_millis(250)) => {} }
        }
      }
      Err(error) => {
        tracing::error!(%error, "Angos restart failed");
        return;
      }
    }
  }
}

async fn stop_child(child: &mut Child, timeout: Duration) {
  #[cfg(unix)]
  if let Some(pid) = child.id() {
    // SAFETY: pid is the live child PID returned by Tokio's process handle.
    unsafe {
      libc::kill(pid as libc::pid_t, libc::SIGTERM);
    }
    if tokio::time::timeout(timeout, child.wait()).await.is_ok() {
      return;
    }
  }
  let _ = child.kill().await;
  let _ = child.wait().await;
}
