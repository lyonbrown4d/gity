use super::{command, keys, transfer};
use crate::AppState;
use russh::{
    Channel, ChannelId,
    server::{self, Msg, Session, VerifiedPublicKey},
};
use std::collections::HashMap;

pub(super) struct Connection {
    state: AppState,
    fingerprint: Option<String>,
    channels: HashMap<ChannelId, Channel<Msg>>,
    tasks: HashMap<ChannelId, tokio::task::JoinHandle<()>>,
    authenticated: std::sync::Arc<tokio::sync::Notify>,
    disconnect: tokio_util::sync::CancellationToken,
    protocol: Option<String>,
}
impl Connection {
    pub fn new(
        state: AppState,
        authenticated: std::sync::Arc<tokio::sync::Notify>,
        disconnect: tokio_util::sync::CancellationToken,
    ) -> Self {
        Self {
            state,
            fingerprint: None,
            channels: HashMap::new(),
            tasks: HashMap::new(),
            authenticated,
            disconnect,
            protocol: None,
        }
    }
}
impl Drop for Connection {
    fn drop(&mut self) {
        for task in self.tasks.values() {
            task.abort();
        }
    }
}
impl server::Handler for Connection {
    type Error = anyhow::Error;
    async fn auth_publickey(
        &mut self,
        user: &str,
        key: VerifiedPublicKey<'_>,
    ) -> Result<server::Auth, Self::Error> {
        if user != "git" {
            return Ok(server::Auth::reject());
        }
        let fingerprint = key
            .public_key
            .fingerprint(russh::keys::HashAlg::Sha256)
            .to_string();
        if tokio::time::timeout(
            std::time::Duration::from_secs(5),
            keys::authorized(&self.state, &fingerprint),
        )
        .await??
        .is_empty()
        {
            return Ok(server::Auth::reject());
        }
        self.fingerprint = Some(fingerprint);
        self.authenticated.notify_one();
        Ok(server::Auth::Accept)
    }
    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        reply: server::ChannelOpenHandle,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.tasks.retain(|_, task| !task.is_finished());
        if self.fingerprint.is_none() || !self.channels.is_empty() || !self.tasks.is_empty() {
            reply
                .reject(russh::ChannelOpenFailure::AdministrativelyProhibited)
                .await;
        } else {
            self.protocol = None;
            self.channels.insert(channel.id(), channel);
            reply.accept().await;
        }
        Ok(())
    }
    async fn env_request(
        &mut self,
        channel: ChannelId,
        _name: &str,
        _value: &str,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if self.channels.contains_key(&channel)
            && _name == "GIT_PROTOCOL"
            && ["version=1", "version=2"].contains(&_value)
        {
            self.protocol = Some(_value.into());
            session.channel_success(channel)?;
        } else {
            session.channel_failure(channel)?;
        }
        Ok(())
    }
    async fn shell_request(
        &mut self,
        id: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.channels.remove(&id);
        if let Some(task) = self.tasks.remove(&id) {
            task.abort();
        }
        session.channel_failure(id)?;
        session.exit_status_request(id, 1)?;
        session.eof(id)?;
        session.close(id)?;
        Ok(())
    }
    async fn subsystem_request(
        &mut self,
        id: ChannelId,
        _name: &str,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.shell_request(id, session).await
    }
    async fn pty_request(
        &mut self,
        id: ChannelId,
        _term: &str,
        _cols: u32,
        _rows: u32,
        _width: u32,
        _height: u32,
        _modes: &[(russh::Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.channel_failure(id)?;
        Ok(())
    }
    async fn channel_close(
        &mut self,
        id: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.channels.remove(&id);
        if let Some(task) = self.tasks.remove(&id) {
            task.abort();
        }
        Ok(())
    }
    async fn exec_request(
        &mut self,
        id: ChannelId,
        raw: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let Some(channel) = self.channels.remove(&id) else {
            session.channel_failure(id)?;
            return Ok(());
        };
        let result = async {
            let (service, path) = command::parse(raw)?;
            let fingerprint = self
                .fingerprint
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("public key authentication required"))?;
            tokio::time::timeout(
                std::time::Duration::from_secs(10),
                transfer::authorize(&self.state, fingerprint, &path, service),
            )
            .await?
        }
        .await;
        match result {
            Ok(mut authorized) => {
                authorized.protocol = self.protocol.take();
                session.channel_success(id)?;
                let state = self.state.clone();
                let handle = session.handle();
                let disconnect = self.disconnect.clone();
                self.tasks.insert(
                    id,
                    tokio::spawn(async move {
                        tokio::select! {
                            _=disconnect.cancelled()=>{},
                            _=transfer::run(state,channel,handle,authorized)=>{},
                        }
                    }),
                );
            }
            Err(error) => {
                tracing::debug!(error=%error,"SSH Git command denied");
                session.channel_failure(id)?;
                session.extended_data(id, 1, b"Git operation denied\n".as_slice())?;
                session.exit_status_request(id, 1)?;
                session.eof(id)?;
                session.close(id)?;
            }
        }
        Ok(())
    }
}
