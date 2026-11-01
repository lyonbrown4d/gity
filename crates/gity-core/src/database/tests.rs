use super::*;
#[tokio::test]
async fn migrations_are_idempotent_and_counters_isolated() {
    let db = Db::connect("sqlite::memory:").await.unwrap();
    db.migrate().await.unwrap();
    db.migrate().await.unwrap();
    assert!(
        db.one("SELECT * FROM users", vec![])
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        db.insert("users; DROP TABLE users", Map::new())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn audit_snapshots_exclude_secrets_and_fail_atomically() {
    let db = Db::connect("sqlite::memory:").await.unwrap();
    db.migrate().await.unwrap();
    db.insert("organizations",serde_json::json!({"id":1,"name":"Team","path_key":"team","full_path":"team","visibility":"private"}).as_object().unwrap().clone()).await.unwrap();
    db.insert("projects",serde_json::json!({"id":1,"organization_id":1,"name":"Project","path_key":"project","full_path":"team/project","visibility":"private","default_branch":"main","status":"active"}).as_object().unwrap().clone()).await.unwrap();
    let fields=serde_json::json!({"project_id":1,"key":"SECRET","value":"never-copy-this-secret","masked":true,"protected":false}).as_object().unwrap().clone();
    audit_context(Some(42), db.insert("project_ci_variables", fields.clone()))
        .await
        .unwrap();
    let audit = db
        .one("SELECT * FROM project_ci_variable_audits", vec![])
        .await
        .unwrap()
        .unwrap();
    assert!(audit.get("value").is_none());
    assert!(!audit.to_string().contains("never-copy-this-secret"));
    let revision = db
        .one("SELECT actor FROM audit_revisions", vec![])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(revision["actor"], "42");
    db.connection
        .execute_unprepared("DROP TABLE project_ci_variable_audits")
        .await
        .unwrap();
    let mut other = fields;
    other.insert("key".into(), serde_json::json!("ROLLBACK"));
    assert!(db.insert("project_ci_variables", other).await.is_err());
    assert!(
        db.one(
            "SELECT id FROM project_ci_variables WHERE key='ROLLBACK'",
            vec![]
        )
        .await
        .unwrap()
        .is_none()
    );
}
