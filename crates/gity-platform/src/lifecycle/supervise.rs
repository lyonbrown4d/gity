use super::{PlatformOptions, PlatformRuntime};
use fluxdi::Shared;
use std::{sync::Arc, time::Duration};
use tokio::{net::TcpListener, task::JoinSet};

pub(super) async fn run(
  runtime: Shared<PlatformRuntime>,
  options: PlatformOptions,
  http: Option<TcpListener>,
  ssh: Option<(TcpListener, Arc<russh::server::Config>)>,
) -> anyhow::Result<()> {
  let mut tasks = JoinSet::<anyhow::Result<()>>::new();
  if let Some(listener) = http {
    let state = runtime.state.clone();
    let stop = runtime.stop.clone();
    tasks.spawn(async move {
      tracing::info!(address=%listener.local_addr()?, "Gity HTTP listener started");
      axum::serve(listener, crate::router(state))
        .with_graceful_shutdown(stop.cancelled_owned())
        .await?;
      Ok(())
    });
  }
  if let Some((listener, config)) = ssh {
    let state = runtime.state.clone();
    let stop = runtime.stop.clone();
    tasks.spawn(async move { crate::ssh::run(state, listener, stop, config).await });
  }
  if options.scheduler {
    let state = runtime.state.clone();
    let stop = runtime.stop.clone();
    tasks.spawn(async move {
      tokio::select! { result = scheduler(state) => result, _ = stop.cancelled() => Ok(()) }
    });
  }
  if options.maintenance {
    let state = runtime.state.clone();
    let stop = runtime.stop.clone();
    tasks.spawn(async move { tokio::select! { result = crate::maintenance::run(state) => result, _ = stop.cancelled() => Ok(()) } });
  }
  let outcome = tokio::select! {
      _ = runtime.stop.cancelled() => Ok(()),
      finished = tasks.join_next(), if !tasks.is_empty() => {
          match finished {
              Some(Ok(Ok(()))) => Err(anyhow::anyhow!("application component stopped unexpectedly")),
              Some(Ok(Err(error))) => Err(error),
              Some(Err(error)) => Err(error.into()),
              None => Ok(()),
          }
      }
  };
  if let Err(error) = &outcome {
    runtime.failure.send_replace(Some(error.to_string()));
  }
  runtime.stop.cancel();
  if tokio::time::timeout(Duration::from_secs(10), async {
    while tasks.join_next().await.is_some() {}
  })
  .await
  .is_err()
  {
    tasks.abort_all();
    while tasks.join_next().await.is_some() {}
  }
  outcome
}

async fn scheduler(state: crate::AppState) -> anyhow::Result<()> {
  let mut interval = tokio::time::interval(Duration::from_secs(1));
  interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
  loop {
    interval.tick().await;
    match crate::ci::worker_tick(&state).await {
      Err(error) => {
        metrics::counter!("gity_scheduler_ticks_total", "result"=>"failure").increment(1);
        tracing::error!(error=%error, "pipeline scheduler tick failed");
      }
      Ok(_) => metrics::counter!("gity_scheduler_ticks_total", "result"=>"success").increment(1),
    }
  }
}
