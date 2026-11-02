use super::test_support::{fixture, request};
use super::*;
use axum::http::StatusCode;

async fn send(s: &AppState, method: &str, path: &str, body: Value) -> (StatusCode, Value) {
  request(s, method, path, "alice", body).await
}

async fn status(s: &AppState, method: &str, path: &str, body: Value) -> StatusCode {
  send(s, method, path, body).await.0
}

async fn data(s: &AppState, method: &str, path: &str, body: Value) -> Value {
  send(s, method, path, body).await.1["data"].clone()
}

fn id_of(envelope: &Value) -> String {
  envelope["data"]["id"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn label_crud_dedupes_titles_and_rejects_conflicts() {
  let (_dir, s) = fixture().await;
  let labels = "/projects/20/labels";
  let (code, created) = send(&s, "POST", labels, json!({"title":" bug "})).await;
  assert_eq!(code, StatusCode::OK);
  assert_eq!(created["data"]["title"], "bug");
  assert_eq!(created["data"]["color"], "#428BCA");
  assert_eq!(created["data"]["description"], Value::Null);
  assert_eq!(created["data"]["project_id"], "20");
  assert!(created["data"]["id"].is_string());
  let duplicate = json!({"title":"bug"});
  assert_eq!(
    status(&s, "POST", labels, duplicate).await,
    StatusCode::BAD_REQUEST
  );
  let blank = json!({"title":" "});
  assert_eq!(
    status(&s, "POST", labels, blank).await,
    StatusCode::BAD_REQUEST
  );
  let (code, other) = send(&s, "POST", "/projects/30/labels", json!({"title":"bug"})).await;
  assert_eq!(code, StatusCode::OK);
  assert_ne!(other["data"]["id"], created["data"]["id"]);
  let listed = data(&s, "GET", labels, json!({})).await;
  assert_eq!(listed, json!([created["data"]]));
  let path = format!("{labels}/{}", id_of(&created));
  let update = json!({"title":"defect","color":"#FFFFFF","description":"broken"});
  let (code, patched) = send(&s, "PATCH", &path, update).await;
  assert_eq!(code, StatusCode::OK);
  assert_eq!(patched["data"]["title"], "defect");
  assert_eq!(patched["data"]["color"], "#FFFFFF");
  assert_eq!(patched["data"]["description"], "broken");
  assert_eq!(patched["data"]["id"], created["data"]["id"]);
  let blank = json!({"color":" "});
  assert_eq!(
    status(&s, "PATCH", &path, blank).await,
    StatusCode::BAD_REQUEST
  );
  let second = json!({"title":"second","description":"x"});
  let (_, second) = send(&s, "POST", labels, second).await;
  let second_path = format!("{labels}/{}", id_of(&second));
  let conflict = json!({"title":"defect"});
  assert_eq!(
    status(&s, "PATCH", &second_path, conflict).await,
    StatusCode::BAD_REQUEST
  );
  let cleared = data(&s, "PATCH", &second_path, json!({"description":null})).await;
  assert_eq!(cleared["description"], Value::Null);
  assert_eq!(cleared["title"], "second");
  assert_eq!(
    status(&s, "DELETE", &second_path, json!({})).await,
    StatusCode::OK
  );
  assert_eq!(
    data(&s, "GET", labels, json!({}))
      .await
      .as_array()
      .unwrap()
      .len(),
    1
  );
  assert_eq!(
    status(&s, "DELETE", &second_path, json!({})).await,
    StatusCode::NOT_FOUND
  );
}

#[tokio::test]
async fn issue_labels_resolve_project_definitions_and_links() {
  let (_dir, s) = fixture().await;
  let path = "/projects/20/issues/1/labels";
  let labels = "/projects/20/labels";
  send(&s, "POST", "/projects/20/issues", json!({"title":"Crash"})).await;
  let attach = json!({
    "labels":[{"name":" bug ","color":"#FF0000"}],
    "names":["bug","ux"],
    "label_names":["bug"],
  });
  let (code, attached) = send(&s, "PATCH", path, attach).await;
  assert_eq!(code, StatusCode::OK);
  let rows = attached["data"].as_array().unwrap();
  assert_eq!(rows.len(), 2);
  assert_eq!(rows[0]["name"], "bug");
  assert_eq!(rows[0]["color"], "#FF0000");
  assert_eq!(rows[0]["issue_id"], "1");
  assert_eq!(rows[0]["description"], Value::Null);
  assert!(rows[0]["id"].is_string());
  assert_eq!(rows[1]["name"], "ux");
  assert_eq!(rows[1]["color"], "#428BCA");
  let defined = data(&s, "GET", labels, json!({})).await;
  assert_eq!(defined[0]["title"], "bug");
  assert_eq!(defined[1]["title"], "ux");
  let detail = send(&s, "GET", "/projects/20/issues/1", json!({})).await.1;
  assert_eq!(detail["data"]["labels"], attached["data"]);
  let list = send(&s, "GET", "/projects/20/issues", json!({})).await.1;
  assert_eq!(list["data"]["items"][0]["labels"], attached["data"]);
  let again = json!({"names":["bug","ux"]});
  assert_eq!(data(&s, "PATCH", path, again).await, attached["data"]);
  let reduced = data(&s, "PATCH", path, json!({"names":["ux"]})).await;
  assert_eq!(reduced.as_array().unwrap().len(), 1);
  assert_eq!(reduced[0]["name"], "ux");
  let kept = data(&s, "GET", labels, json!({})).await;
  assert_eq!(kept.as_array().unwrap().len(), 2);
  let ux = reduced[0]["id"].as_str().unwrap();
  let delete = format!("{labels}/{ux}");
  assert_eq!(
    status(&s, "DELETE", &delete, json!({})).await,
    StatusCode::OK
  );
  assert_eq!(data(&s, "GET", path, json!({})).await, json!([]));
  assert_eq!(
    data(&s, "PATCH", path, json!({"names":[]})).await,
    json!([])
  );
}

#[tokio::test]
async fn label_api_enforces_project_scope_and_permissions() {
  let (_dir, s) = fixture().await;
  let (_, created) = send(&s, "POST", "/projects/20/labels", json!({"title":"bug"})).await;
  let bug = id_of(&created);
  let labels = "/projects/20/labels";
  assert_eq!(
    request(&s, "GET", labels, "outsider", json!({})).await.0,
    StatusCode::FORBIDDEN
  );
  let evil = json!({"title":"evil"});
  assert_eq!(
    request(&s, "POST", labels, "outsider", evil).await.0,
    StatusCode::FORBIDDEN
  );
  let foreign = format!("/projects/30/labels/{bug}");
  assert_eq!(
    status(&s, "PATCH", &foreign, json!({"title":"x"})).await,
    StatusCode::NOT_FOUND
  );
  assert_eq!(
    status(&s, "DELETE", &foreign, json!({})).await,
    StatusCode::NOT_FOUND
  );
  send(&s, "POST", "/projects/20/issues", json!({"title":"Crash"})).await;
  send(&s, "POST", "/projects/30/issues", json!({"title":"Other"})).await;
  let path = "/projects/20/issues/1/labels";
  let (code, attached) = request(&s, "PATCH", path, "bob", json!({"names":["bug"]})).await;
  assert_eq!(code, StatusCode::OK);
  assert_eq!(attached["data"][0]["id"], created["data"]["id"]);
  let other = "/projects/30/issues/1/labels";
  assert_eq!(data(&s, "GET", other, json!({})).await, json!([]));
  assert_eq!(
    data(&s, "GET", "/projects/30/labels", json!({})).await,
    json!([])
  );
}
