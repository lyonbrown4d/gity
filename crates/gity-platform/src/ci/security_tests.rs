use super::test_support::*;
use super::*;

#[tokio::test]
async fn inline_configs_and_manual_jobs_cannot_assert_protected_secret_trust() {
  let (_dir, s, app, owner, p) = fixture().await;
  let fields = json!({"project_id":p,"key":"PROTECTED_SECRET","value":"protected-test-secret","masked":1,"protected":1});
  s.db
    .insert("project_ci_variables", fields.as_object().unwrap().clone())
    .await
    .unwrap();
  let fields = json!({"username":"developer","display_name":"Developer","email":"developer@example.test","is_super_admin":0});
  let developer = s
    .db
    .insert("users", fields.as_object().unwrap().clone())
    .await
    .unwrap();
  let fields = json!({"project_id":p,"user_id":id(&developer),"role":"developer"});
  s.db
    .insert("project_members", fields.as_object().unwrap().clone())
    .await
    .unwrap();
  let developer_token = "developer-test-session";
  s.db
    .execute(
      "INSERT INTO auth_sessions(token_hash,user_id,kind,expires_at,family) VALUES (?,?,?,?,?)",
      vec![
        crate::auth::token_hash(developer_token).into(),
        id(&developer).into(),
        "access".into(),
        worker::BLOCKED.into(),
        "developer-test".into(),
      ],
    )
    .await
    .unwrap();
  let sha = s
    .git
    .resolve_revision(&format!("{p}.git"), "main", "")
    .await
    .unwrap();
  let (status, _) = request(
        &app,
        "POST",
        &format!("/projects/{p}/pipelines"),
        Some(developer_token),
        json!({"source":"push","ref_name":"main","commit_sha":sha,"config_content":"jobs: {pass: {script: 'echo pass'}}"}),
    )
    .await;
  assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
  let forged=post(&app,&format!("/projects/{p}/pipelines"),Some(developer_token),json!({"ref_name":"main","commit_sha":sha,"config_source":".gity-ci.yml","config_content":"jobs: {extract: {script: 'printf %s $PROTECTED_SECRET'}}"})).await;
  assert_eq!(forged["pipeline"]["config_source"], ".gity-ci.yml");
  let registered = post(
    &app,
    &format!("/projects/{p}/runners"),
    Some(&owner),
    json!({"name":"runner","tags":"linux"}),
  )
  .await;
  let token = text(&registered, "token");
  let claim = post(&app, "/runners/jobs/claim", None, json!({"token":token})).await;
  let data: Value = serde_json::from_str(text(&claim["job"], "payload")).unwrap();
  assert_eq!(data["trusted_config"], false);
  assert!(data["env"].get("PROTECTED_SECRET").is_none());
  let manual = json!({"script":["printf %s $PROTECTED_SECRET"],"ref_name":"main","commit_sha":sha,"trusted_config":true});
  post(
    &app,
    &format!("/projects/{p}/jobs"),
    Some(developer_token),
    json!({"kind":"script","payload":manual.to_string()}),
  )
  .await;
  let claimed = post(&app, "/runners/jobs/claim", None, json!({"token":token})).await;
  let data: Value = serde_json::from_str(text(&claimed["job"], "payload")).unwrap();
  assert_eq!(data["trusted_config"], false);
  assert!(data["env"].get("PROTECTED_SECRET").is_none());
}

#[tokio::test]
async fn protected_patterns_require_trusted_yaml_and_the_matching_branch_tip() {
  let (_dir, s, _app, _owner, project) = fixture().await;
  let variable = json!({"project_id":project,"key":"RELEASE_SECRET","value":"release-secret","masked":1,"protected":1});
  s.db
    .insert(
      "project_ci_variables",
      variable.as_object().unwrap().clone(),
    )
    .await
    .unwrap();
  let protection = json!({"project_id":project,"branch_name":"release/*","rule_type":"pattern","push_access_level":"maintainer","merge_access_level":"maintainer","require_merge_request":0,"require_pipeline_success":0,"allow_force_push":0,"allow_delete":0});
  s.db
    .insert(
      "project_branch_protections",
      protection.as_object().unwrap().clone(),
    )
    .await
    .unwrap();
  s.git
    .create_branch(&format!("{project}.git"), "release/v1", "main")
    .await
    .unwrap();
  let original = s
    .git
    .resolve_revision(&format!("{project}.git"), "main", "")
    .await
    .unwrap();
  let mut data =
    json!({"trusted_config":true,"ref_name":"release/v1","commit_sha":original,"yaml_env":{}});
  let (env, masked) = variables::environment(&s, project, &data).await.unwrap();
  assert_eq!(env.get("RELEASE_SECRET").unwrap(), "release-secret");
  assert_eq!(masked, ["release-secret"]);
  data["trusted_config"] = json!(false);
  assert!(
    !variables::environment(&s, project, &data)
      .await
      .unwrap()
      .0
      .contains_key("RELEASE_SECRET")
  );
  s.git
    .create_branch(&format!("{project}.git"), "feature", "main")
    .await
    .unwrap();
  s.git
    .create_file_commit(
      &format!("{project}.git"),
      gity_git::CreateFileCommitInput {
        branch_name: "feature".into(),
        file_path: "untrusted.txt".into(),
        content: "untrusted".into(),
        message: "untrusted".into(),
        author_name: "Developer".into(),
        author_email: "developer@example.test".into(),
      },
    )
    .await
    .unwrap();
  data["trusted_config"] = json!(true);
  data["commit_sha"] = json!(
    s.git
      .resolve_revision(&format!("{project}.git"), "feature", "")
      .await
      .unwrap()
  );
  assert!(
    !variables::environment(&s, project, &data)
      .await
      .unwrap()
      .0
      .contains_key("RELEASE_SECRET"),
    "naming a protected branch cannot authorize an unrelated commit"
  );
}

#[tokio::test]
async fn project_tokens_separate_job_read_write_and_runner_administration() {
  let (_dir, s, app, _owner, project) = fixture().await;
  let owner = s
    .db
    .one("SELECT id FROM users WHERE username='owner'", vec![])
    .await
    .unwrap()
    .unwrap();
  for (raw, scope) in [
    ("read-ci-token", "read_api"),
    ("write-ci-token", "write_api"),
  ] {
    let fields = json!({"project_id":project,"name":scope,"username":"ci-bot","token":format!("sha256:{}",crate::auth::token_hash(raw)),"kind":"access","scopes":scope,"created_by_user_id":id(&owner),"expires_at":worker::BLOCKED,"revoked_at":null});
    s.db
      .insert("project_access_tokens", fields.as_object().unwrap().clone())
      .await
      .unwrap();
  }
  let status = |expected, actual| assert_eq!(expected, actual);
  for token in ["read-ci-token", "write-ci-token"] {
    status(
      axum::http::StatusCode::OK,
      request(
        &app,
        "GET",
        &format!("/projects/{project}/jobs"),
        Some(token),
        json!({}),
      )
      .await
      .0,
    );
    status(
      axum::http::StatusCode::OK,
      request(
        &app,
        "GET",
        &format!("/projects/{project}/runners"),
        Some(token),
        json!({}),
      )
      .await
      .0,
    );
    status(
      axum::http::StatusCode::FORBIDDEN,
      request(
        &app,
        "GET",
        &format!("/projects/{}/jobs", project + 1),
        Some(token),
        json!({}),
      )
      .await
      .0,
    );
    assert!(
      !request(
        &app,
        "POST",
        &format!("/projects/{project}/runners"),
        Some(token),
        json!({"name":"escalate"})
      )
      .await
      .0
      .is_success()
    );
  }
  status(
    axum::http::StatusCode::FORBIDDEN,
    request(
      &app,
      "POST",
      &format!("/projects/{project}/jobs"),
      Some("read-ci-token"),
      json!({"kind":"noop","payload":"{}"}),
    )
    .await
    .0,
  );
  status(
    axum::http::StatusCode::OK,
    request(
      &app,
      "POST",
      &format!("/projects/{project}/jobs"),
      Some("write-ci-token"),
      json!({"kind":"noop","payload":"{}"}),
    )
    .await
    .0,
  );
}
