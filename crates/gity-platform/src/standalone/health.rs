use fluxdi::{Injector, Shared};
use gity_core::Settings;

/// Monitor the lifecycle handles rather than accepting a healthy HTTP listener
/// while a required sibling component has stopped.
pub async fn monitor(injector: Shared<Injector>, settings: &Settings, full: bool) -> String {
  let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
  loop {
    interval.tick().await;
    if full && settings.components.runner.enabled {
      match injector.try_resolve::<super::runner::RunnerHealth>() {
        Ok(health) if health.is_healthy() => {}
        _ => return "Runner component stopped".into(),
      }
    }
    #[cfg(feature = "oci")]
    {
      if settings.components.authority.enabled {
        match injector.try_resolve::<gity_authority::AuthorityHealth>() {
          Ok(health) if health.is_healthy() => {}
          _ => return "identity authority stopped".into(),
        }
      }
      if full && settings.components.registry.enabled {
        match injector.try_resolve::<gity_runtime::AngosProcess>() {
          Ok(process) if !process.has_failed() => {}
          _ => return "OCI engine exhausted its restart policy".into(),
        }
        match injector.try_resolve::<gity_oci::OciGatewayHealth>() {
          Ok(health) if health.is_healthy() => {}
          _ => return "OCI gateway stopped".into(),
        }
      }
    }
  }
}
