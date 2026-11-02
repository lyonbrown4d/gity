use std::future::Future;
tokio::task_local! {static ACTOR:Option<i64>;}

pub async fn context<F: Future>(actor: Option<i64>, future: F) -> F::Output {
  ACTOR.scope(actor, future).await
}
pub fn actor() -> Option<i64> {
  ACTOR.try_with(|value| *value).ok().flatten()
}
