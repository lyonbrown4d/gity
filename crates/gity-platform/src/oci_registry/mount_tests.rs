use super::PlatformOciAuthorizer;
use axum::{
  Router,
  body::Body,
  extract::State,
  http::{Request, StatusCode},
  response::Response,
};
use gity_entities::{project_members, project_registry_settings};
use gity_oci::{OciGateway, OciSettings, UpstreamPolicy};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::{
  sync::{Arc, Mutex},
  time::Duration,
};

type Calls = Arc<Mutex<Vec<reqwest::Url>>>;

async fn upstream(State(calls): State<Calls>, request: Request<Body>) -> Response {
  assert!(!request.headers().contains_key("authorization"));
  let uri = reqwest::Url::parse(&format!("http://storage{}", request.uri())).unwrap();
  let mount = uri.query_pairs().any(|(key, _)| key == "mount");
  calls.lock().unwrap().push(uri);
  let location = if mount {
    "/v2/projects/20/repositories/image/blobs/sha256:abc"
  } else {
    "/v2/projects/20/repositories/image/blobs/uploads/new-session"
  };
  Response::builder()
    .status(if mount { 201 } else { 202 })
    .header("location", location)
    .header("docker-upload-uuid", "new-session")
    .body(Body::empty())
    .unwrap()
}

async fn serve(app: Router) -> (String, tokio::task::JoinHandle<()>) {
  let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
  let origin = format!("http://{}", listener.local_addr().unwrap());
  let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
  (origin, task)
}

async fn mount(
  client: &reqwest::Client,
  origin: &str,
  source: &str,
  user: &str,
) -> reqwest::Response {
  client
    .post(format!("{origin}/v2/team/p20/image/blobs/uploads/"))
    .query(&[
      ("mount", "sha256:abc"),
      ("from", source),
      ("keep", "a+b value"),
      ("mount", "sha256:def"),
      ("from", "untrusted/duplicate"),
    ])
    .bearer_auth(user)
    .send()
    .await
    .unwrap()
}

fn assert_upload(response: &reqwest::Response, calls: &Calls) {
  assert_eq!(response.status(), StatusCode::ACCEPTED);
  assert_eq!(response.headers()["docker-upload-uuid"], "new-session");
  assert_eq!(
    response.headers()["location"],
    "/v2/team/p20/image/blobs/uploads/new-session"
  );
  let calls = calls.lock().unwrap();
  let latest = calls.last().unwrap();
  assert_eq!(
    latest.path(),
    "/v2/projects/20/repositories/image/blobs/uploads/"
  );
  assert_eq!(
    latest.query_pairs().collect::<Vec<_>>(),
    [("keep".into(), "a+b value".into())]
  );
}

#[tokio::test]
async fn mount_router_falls_back_for_inaccessible_or_mirror_sources_without_relaxing_target_acl() {
  let (_dir, mut state) = crate::features::test_support::fixture().await;
  let mut settings = (*state.settings).clone();
  settings.components.registry.mirrors.insert(
    "cache".into(),
    gity_core::MirrorUpstreamConfig {
      host: "registry.example".into(),
      allow_all_projects: true,
      ..Default::default()
    },
  );
  state.settings = Arc::new(settings);
  let calls: Calls = Arc::default();
  let (storage, upstream_task) =
    serve(Router::new().fallback(upstream).with_state(calls.clone())).await;
  let gateway = OciGateway::new(
    OciSettings {
      upstream: storage,
      request_timeout: Duration::from_secs(5),
      upstream_policy: UpstreamPolicy::default(),
      mirror: None,
    },
    Arc::new(PlatformOciAuthorizer::new(state.clone())),
  )
  .unwrap();
  let (origin, gateway_task) = serve(gity_oci::router(gateway)).await;
  let client = reqwest::Client::new();
  let response = mount(&client, &origin, "team/p30/source", "alice").await;
  assert_eq!(response.status(), StatusCode::CREATED);
  let recorded = calls.lock().unwrap().last().unwrap().clone();
  let sources = recorded
    .query_pairs()
    .filter(|(key, _)| key == "from")
    .map(|(_, value)| value.into_owned())
    .collect::<Vec<_>>();
  assert_eq!(
    sources,
    [
      "projects/30/repositories/source",
      "projects/30/repositories/source"
    ]
  );

  project_members::Entity::delete_many()
    .filter(project_members::Column::ProjectId.eq(30))
    .filter(project_members::Column::UserId.eq(1))
    .exec(&state.db.connection)
    .await
    .unwrap();
  assert_upload(
    &mount(&client, &origin, "team/p30/source", "alice").await,
    &calls,
  );
  assert_upload(
    &mount(&client, &origin, "team/deleted/source", "alice").await,
    &calls,
  );
  let count = calls.lock().unwrap().len();
  assert_eq!(
    mount(&client, &origin, "team/p30/source", "outsider")
      .await
      .status(),
    StatusCode::FORBIDDEN
  );
  assert_eq!(
    calls.lock().unwrap().len(),
    count,
    "denied target reached storage"
  );

  project_registry_settings::ActiveModel {
    project_id: Set(30),
    mode: Set("mirror".into()),
    mirror_alias: Set(Some("cache".into())),
    mirror_repository_prefix: Set("library".into()),
    updated_at: Set(chrono::Utc::now().naive_utc()),
    ..Default::default()
  }
  .insert(&state.db.connection)
  .await
  .unwrap();
  assert_upload(
    &mount(&client, &origin, "team/p30/source", "bob").await,
    &calls,
  );
  project_registry_settings::ActiveModel {
    project_id: Set(30),
    mirror_alias: Set(None),
    ..Default::default()
  }
  .update(&state.db.connection)
  .await
  .unwrap();
  let count = calls.lock().unwrap().len();
  assert_eq!(
    mount(&client, &origin, "team/p30/source", "bob")
      .await
      .status(),
    StatusCode::SERVICE_UNAVAILABLE
  );
  assert_eq!(
    calls.lock().unwrap().len(),
    count,
    "source infrastructure error was swallowed"
  );
  gateway_task.abort();
  upstream_task.abort();
}
