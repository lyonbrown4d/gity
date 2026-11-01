use crate::GitNodeSettings;
use gity_git::GitService;
use gity_git_wire::{
  GitOperation, RepositoryKey,
  wire::{self, git_storage_server::GitStorage},
};
use std::future::Future;
use std::{
  collections::HashMap,
  sync::{
    Arc, Weak,
    atomic::{AtomicU64, Ordering},
  },
  time::Duration,
};
use tokio::{
  sync::{Mutex, OwnedMutexGuard, oneshot},
  task::JoinHandle,
  time::timeout_at,
};
use tokio_util::sync::CancellationToken;
use tonic::{Request, Response, Status};

struct ActiveWorker {
  stop: CancellationToken,
  task: JoinHandle<()>,
}

#[derive(Clone)]
pub struct GitNodeService {
  pub(crate) settings: Arc<GitNodeSettings>,
  pub(crate) git: GitService,
  locks: Arc<Mutex<HashMap<String, Weak<Mutex<()>>>>>,
  workers: Arc<Mutex<HashMap<u64, ActiveWorker>>>,
  next_worker: Arc<AtomicU64>,
  shutdown: CancellationToken,
}
impl GitNodeService {
  pub fn new(settings: GitNodeSettings) -> anyhow::Result<Self> {
    settings.validate()?;
    Ok(Self {
      git: GitService::new(&settings.repo_root, &settings.git_bin),
      settings: Arc::new(settings),
      locks: Arc::new(Mutex::new(HashMap::new())),
      workers: Arc::new(Mutex::new(HashMap::new())),
      next_worker: Arc::new(AtomicU64::new(1)),
      shutdown: CancellationToken::new(),
    })
  }
  pub(crate) async fn spawn_worker<F, Fut>(&self, worker: F) -> Result<(), Status>
  where
    F: FnOnce(CancellationToken) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    let mut workers = self.workers.lock().await;
    workers.retain(|_, active| !active.task.is_finished());
    if self.shutdown.is_cancelled() {
      return Err(Status::unavailable("Git node is stopping"));
    }
    let id = self.next_worker.fetch_add(1, Ordering::Relaxed);
    let stop = self.shutdown.child_token();
    let task_stop = stop.clone();
    let task = tokio::spawn(worker(task_stop));
    workers.insert(id, ActiveWorker { stop, task });
    Ok(())
  }
  pub(crate) async fn stop_workers(&self) {
    self.shutdown.cancel();
    let mut workers: Vec<_> = self
      .workers
      .lock()
      .await
      .drain()
      .map(|(_, task)| task)
      .collect();
    for worker in &workers {
      worker.stop.cancel();
    }
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let mut timed_out = false;
    for worker in &mut workers {
      if timeout_at(deadline, &mut worker.task).await.is_err() {
        timed_out = true;
        break;
      }
    }
    if timed_out {
      for worker in &workers {
        worker.task.abort();
      }
    }
    for worker in workers {
      let _ = worker.task.await;
    }
  }
  pub(crate) async fn write_lock(&self, repo: &str) -> OwnedMutexGuard<()> {
    let lock = {
      let mut locks = self.locks.lock().await;
      if locks.len() >= 4096 {
        locks.retain(|_, value| value.strong_count() > 0);
      }
      if let Some(lock) = locks.get(repo).and_then(Weak::upgrade) {
        lock
      } else {
        let lock = Arc::new(Mutex::new(()));
        locks.insert(repo.to_owned(), Arc::downgrade(&lock));
        lock
      }
    };
    lock.lock_owned().await
  }
  pub(crate) fn allow_write(&self) -> Result<(), Status> {
    if self.settings.read_only {
      Err(Status::failed_precondition("Git storage node is read only"))
    } else {
      Ok(())
    }
  }
  pub(crate) fn authorize<T>(&self, request: &Request<T>) -> Result<(), Status> {
    use sha2::{Digest, Sha256};
    use subtle::ConstantTimeEq;
    let given = request
      .metadata()
      .get("authorization")
      .and_then(|v| v.to_str().ok())
      .and_then(|v| v.strip_prefix("Bearer "))
      .unwrap_or_default();
    let expected = Sha256::digest(self.settings.token.as_bytes());
    let actual = Sha256::digest(given.as_bytes());
    if bool::from(expected.ct_eq(&actual)) {
      Ok(())
    } else {
      Err(Status::unauthenticated(
        "invalid internal service credential",
      ))
    }
  }
}
#[tonic::async_trait]
impl GitStorage for GitNodeService {
  async fn execute(
    &self,
    request: Request<wire::ExecuteRequest>,
  ) -> Result<Response<wire::ExecuteResponse>, Status> {
    self.authorize(&request)?;
    let request = request.into_inner();
    let repo = RepositoryKey::try_from(request.repository_key)
      .map_err(|_| Status::invalid_argument("invalid repository key"))?;
    let operation: GitOperation = serde_json::from_slice(&request.operation_json)
      .map_err(|_| Status::invalid_argument("invalid typed Git operation"))?;
    let is_replica_sync = matches!(&operation, GitOperation::SyncReplica { .. });
    let service = self.clone();
    let needs_lock = operation.is_write() || operation.is_control();
    let (send, receive) = oneshot::channel();
    self
      .spawn_worker(move |stop| async move {
        // Keep the lock until a mutation really finishes, even when its caller disconnects.
        let result = async {
          let _guard = if needs_lock {
            Some(service.write_lock(repo.as_str()).await)
          } else {
            None
          };
          service
            .execute_checked(repo.as_str(), operation, request.epoch, stop.clone())
            .await
        };
        let result = if is_replica_sync {
          result.await
        } else {
          tokio::select! {
            _ = stop.cancelled() => Err(anyhow::anyhow!("Git node is stopping")),
            result = result => result,
          }
        };
        let _ = send.send(result);
      })
      .await?;
    let result = receive
      .await
      .map_err(|_| Status::internal("Git storage worker stopped"))?
      .map_err(|error| {
        tracing::warn!(error=%error, "Git storage operation failed");
        Status::failed_precondition("Git storage operation failed")
      })?;
    Ok(Response::new(wire::ExecuteResponse {
      result_json: serde_json::to_vec(&result)
        .map_err(|_| Status::internal("serialize Git storage result"))?,
    }))
  }
  type TransferStream = tokio_stream::wrappers::ReceiverStream<Result<wire::TransferFrame, Status>>;
  async fn transfer(
    &self,
    request: Request<tonic::Streaming<wire::TransferFrame>>,
  ) -> Result<Response<Self::TransferStream>, Status> {
    self.authorize(&request)?;
    self
      .transfer_stream(request.into_inner())
      .await
      .map(Response::new)
  }
  async fn health(
    &self,
    request: Request<wire::HealthRequest>,
  ) -> Result<Response<wire::HealthResponse>, Status> {
    self.authorize(&request)?;
    Ok(Response::new(wire::HealthResponse {
      node_id: self.settings.id.clone(),
      writable: !self.settings.read_only,
    }))
  }
}
