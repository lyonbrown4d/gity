//! Dedicated, bounded SSH listener backed by russh public-key authentication.
mod command;
mod connection;
mod keys;
mod session;
mod transfer;
mod updates;
use crate::AppState;
pub(crate) use keys::canonical_key;
use std::{sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

pub async fn run(
    state: AppState,
    listener: TcpListener,
    shutdown: CancellationToken,
    config: Arc<russh::server::Config>,
) -> anyhow::Result<()> {
    tracing::info!(address=%listener.local_addr()?,"Gity SSH listener started");
    let permits = Arc::new(tokio::sync::Semaphore::new(
        state.settings.ssh.max_connections,
    ));
    let mut tasks = tokio::task::JoinSet::new();
    loop {
        let permit = tokio::select! {
            _=shutdown.cancelled()=>break,
            permit=permits.clone().acquire_owned()=>permit?,
        };
        let (stream, peer) = tokio::select! {
            _=shutdown.cancelled()=>break,
            incoming=listener.accept()=>incoming?,
        };
        let config = config.clone();
        let state = state.clone();
        let cancel = shutdown.clone();
        tasks.spawn(async move {
            let _permit = permit;
            if let Err(error) = session::run(state, config, stream, cancel).await {
                tracing::debug!(%peer,error=%error,"SSH session ended");
            }
        });
        while tasks.try_join_next().is_some() {}
    }
    while tasks.join_next().await.is_some() {}
    Ok(())
}

pub fn configuration(state: &AppState) -> anyhow::Result<Arc<russh::server::Config>> {
    let settings = &state.settings.ssh;
    let config = Arc::new(russh::server::Config {
        keys: vec![keys::host_key(&settings.host_key)?],
        methods: russh::MethodSet::from(&[russh::MethodKind::PublicKey][..]),
        max_auth_attempts: 5,
        inactivity_timeout: Some(Duration::from_secs(settings.operation_timeout_seconds)),
        auth_rejection_time: Duration::from_millis(300),
        auth_rejection_time_initial: Some(Duration::ZERO),
        ..Default::default()
    });
    Ok(config)
}
