use crate::{auth_tests::request, features::test_support::fixture};
use axum::http::StatusCode;
use serde_json::{Value, json};

async fn row(state: &crate::AppState, table: &str, id: i64) -> Value {
    state
        .db
        .one(
            &format!("SELECT * FROM {table} WHERE id=?"),
            vec![id.into()],
        )
        .await
        .unwrap()
        .unwrap()
}

#[tokio::test]
async fn namespace_rename_preserves_git_ci_descendants_and_atomic_conflict_rollback() {
    let (_root, state) = fixture().await;
    state
        .db
        .insert(
            "organization_members",
            json!({"organization_id":10,"user_id":1,"role":"owner"})
                .as_object()
                .unwrap()
                .clone(),
        )
        .await
        .unwrap();
    for (id, key, path) in [
        (11, "sub", "team/sub"),
        (12, "team-extra", "team-extra"),
        (13, "newXteam", "newXteam"),
        (14, "occupied", "occupied"),
    ] {
        state
            .db
            .insert(
                "organizations",
                json!({"id":id,"name":key,"path_key":key,"full_path":path,"visibility":"private"})
                    .as_object()
                    .unwrap()
                    .clone(),
            )
            .await
            .unwrap();
    }
    state.db.insert("projects",json!({"id":40,"organization_id":11,"name":"Nested","path_key":"nested","full_path":"team/sub/nested","visibility":"private","default_branch":"main","status":"active"}).as_object().unwrap().clone()).await.unwrap();
    state.git.init_bare("20.git", "main").await.unwrap();
    let yaml = "jobs: {test: {script: 'echo test'}}\n";
    state
        .git
        .create_file_commit(
            "20.git",
            gity_git::CreateFileCommitInput {
                branch_name: "main".into(),
                file_path: ".gity-ci.yml".into(),
                content: yaml.into(),
                message: "Seed CI".into(),
                author_name: "Alice".into(),
                author_email: "alice@test.local".into(),
            },
        )
        .await
        .unwrap();
    let sha = state
        .git
        .resolve_revision("20.git", "main", "")
        .await
        .unwrap();
    let (status, body) = request(
        &state,
        "PATCH",
        "/orgs/10",
        "alice",
        json!({"key":"new_team","name":"Renamed"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["data"]["key"], "new_team");
    assert_eq!(
        row(&state, "organizations", 11).await["full_path"],
        "new_team/sub"
    );
    assert_eq!(row(&state, "organizations", 11).await["path_key"], "sub");
    assert_eq!(
        row(&state, "projects", 40).await["full_path"],
        "new_team/sub/nested"
    );
    assert_eq!(
        row(&state, "organizations", 12).await["full_path"],
        "team-extra"
    );
    assert_eq!(
        state
            .git
            .resolve_revision("20.git", "main", "")
            .await
            .unwrap(),
        sha
    );
    let (status, blob) = request(
        &state,
        "GET",
        "/projects/20/repository/blob?ref_name=main&path=.gity-ci.yml",
        "alice",
        json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{blob}");
    assert_eq!(blob["data"]["content"], yaml);
    let pipeline = crate::ci::create_push_pipeline(&state, 20, "main", &sha)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(pipeline["pipeline"]["commit_sha"], sha);
    let job = state
        .db
        .one(
            "SELECT payload FROM project_jobs WHERE project_id=?",
            vec![20.into()],
        )
        .await
        .unwrap()
        .unwrap();
    let payload: Value = serde_json::from_str(job["payload"].as_str().unwrap()).unwrap();
    assert_eq!(payload["project_full_path"], "new_team/p20");
    assert!(_root.path().join("repos/20.git").is_dir());
    assert!(!_root.path().join("repos/new_team/p20.git").exists());
    let conflict = request(
        &state,
        "PATCH",
        "/orgs/10",
        "alice",
        json!({"key":"occupied","name":"Must Roll Back"}),
    )
    .await;
    assert_eq!(conflict.0, StatusCode::CONFLICT, "{}", conflict.1);
    assert_eq!(row(&state, "organizations", 10).await["name"], "Renamed");
    assert_eq!(
        row(&state, "projects", 40).await["full_path"],
        "new_team/sub/nested"
    );
    state.db.insert("projects",json!({"id":50,"organization_id":14,"name":"Collision","path_key":"p20","full_path":"project-collision/p20","visibility":"private","default_branch":"main","status":"active"}).as_object().unwrap().clone()).await.unwrap();
    let conflict = request(
        &state,
        "PATCH",
        "/orgs/10",
        "alice",
        json!({"key":"project-collision","name":"Must Also Roll Back"}),
    )
    .await;
    assert_eq!(conflict.0, StatusCode::CONFLICT, "{}", conflict.1);
    assert_eq!(
        row(&state, "organizations", 10).await["full_path"],
        "new_team"
    );
    assert_eq!(
        row(&state, "organizations", 11).await["full_path"],
        "new_team/sub"
    );
    let (renamed, created) = tokio::join!(
        request(&state, "PATCH", "/orgs/10", "alice", json!({"key":"final"})),
        request(
            &state,
            "POST",
            "/projects",
            "alice",
            json!({"organization_id":10,"key":"concurrent","name":"Concurrent"})
        ),
    );
    assert_eq!(renamed.0, StatusCode::OK, "{}", renamed.1);
    assert_eq!(created.0, StatusCode::OK, "{}", created.1);
    let id = created.1["data"]["id"].as_str().unwrap().parse().unwrap();
    assert_eq!(
        row(&state, "projects", id).await["full_path"],
        "final/concurrent"
    );
    assert!(_root.path().join(format!("repos/{id}.git")).is_dir());
    assert_eq!(
        row(&state, "organizations", 13).await["full_path"],
        "newXteam"
    );
    assert_eq!(row(&state, "projects", 20).await["full_path"], "final/p20");
    assert_eq!(
        crate::repository::repo(&json!({"id":"20","full_path":"ignored"})),
        "20.git"
    );
}
