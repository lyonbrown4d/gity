use chrono::{DateTime, Utc};
use gity_core::{Db, new_id, now};
use gity_entities::{project_jobs, project_pipelines};
use sea_orm::EntityTrait;

const ZERO: &str = "1970-01-02T00:00:00.000Z";

#[tokio::test]
async fn pending_jobs_and_pipelines_decode_zero_time_sentinels() {
  let db = Db::connect("sqlite::memory:").await.unwrap();
  db.migrate().await.unwrap();
  let org = new_id();
  let project = new_id();
  let job = new_id();
  let pipeline = new_id();
  let stamp = now();
  let zero = DateTime::parse_from_rfc3339(ZERO)
    .unwrap()
    .with_timezone(&Utc)
    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

  db.execute(
        "INSERT INTO organizations(id,name,path_key,full_path,visibility,created_at,updated_at) VALUES (?,?,?,?,?,?,?)",
        vec![org.into(), "test".into(), "test".into(), "test".into(), "private".into(), stamp.clone().into(), stamp.clone().into()],
    ).await.unwrap();
  db.execute(
        "INSERT INTO projects(id,organization_id,name,path_key,full_path,visibility,default_branch,status,created_at,updated_at) VALUES (?,?,?,?,?,?,?,?,?,?)",
        vec![project.into(), org.into(), "test".into(), "test".into(), "test/test".into(), "private".into(), "main".into(), "active".into(), stamp.clone().into(), stamp.clone().into()],
    ).await.unwrap();
  db.execute(
        "INSERT INTO project_jobs(id,project_id,kind,status,attempts,max_attempts,run_after,locked_until,created_at,updated_at,started_at,finished_at) VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
        vec![job.into(), project.into(), "script".into(), "pending".into(), 0i64.into(), 3i64.into(), stamp.clone().into(), zero.clone().into(), stamp.clone().into(), stamp.clone().into(), zero.clone().into(), zero.clone().into()],
    ).await.unwrap();
  db.execute(
        "INSERT INTO project_pipelines(id,project_id,iid,name,source,status,config_source,config_content,created_at,updated_at,started_at,finished_at) VALUES (?,?,?,?,?,?,?,?,?,?,?,?)",
        vec![pipeline.into(), project.into(), 1i64.into(), "test".into(), "api".into(), "pending".into(), "database".into(), "jobs: []".into(), stamp.clone().into(), stamp.into(), zero.clone().into(), zero.into()],
    ).await.unwrap();

  let job = project_jobs::Entity::find_by_id(job)
    .one(&db.connection)
    .await
    .unwrap()
    .unwrap();
  let pipeline = project_pipelines::Entity::find_by_id(pipeline)
    .one(&db.connection)
    .await
    .unwrap()
    .unwrap();
  assert_eq!(job.status, "pending");
  assert_eq!(pipeline.status, "pending");
  let expected = DateTime::parse_from_rfc3339(ZERO)
    .unwrap()
    .with_timezone(&Utc)
    .naive_utc();
  assert_eq!(job.started_at, expected);
  assert_eq!(pipeline.finished_at, expected);
}
