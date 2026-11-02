use super::*;

pub(super) fn fixture<'a>(
  config: &'a RunConfig,
  scenario: &'a api::Scenario,
  reporter: &mut Reporter,
) -> Result<Fixture<'a>> {
  let client = Client::builder()
    .timeout(Duration::from_secs(120))
    .build()?;
  let (status, existing) = api::request_json(
    &client,
    config,
    Method::GET,
    &format!("/projects/{}", scenario.project_id),
    Some(&scenario.owner_token),
    None,
  )?;
  api::require_status(status, StatusCode::OK, "read owner organization")?;
  let organization = api::id(
    &api::data(&existing)["organization_id"],
    "project organization",
  )?;
  let (status, project) = api::request_json(
    &client,
    config,
    Method::POST,
    "/projects",
    Some(&scenario.owner_token),
    Some(json!({
      "organization_id":organization,"name":"Merge recovery acceptance","key":format!("merge-{}",scenario.suffix),"visibility":"private","default_branch":"main"
    })),
  )?;
  api::require_status(status, StatusCode::OK, "isolated collaboration project")?;
  let project_path = text(api::data(&project), "full_path")?;
  let project = api::id(&api::data(&project)["id"], "collaboration project")?;
  let username = format!("bb_review_{}", scenario.suffix);
  let password = uuid::Uuid::new_v4().simple().to_string();
  reporter.remember_secret(&password);
  let (status, _) = api::request_json(
    &client,
    config,
    Method::POST,
    "/auth/register",
    None,
    Some(json!({"username":username,"password":password})),
  )?;
  api::require_status(status, StatusCode::OK, "reviewer registration")?;
  let (status, login) = api::request_json(
    &client,
    config,
    Method::POST,
    "/auth/login",
    None,
    Some(json!({"username":username,"password":password})),
  )?;
  api::require_status(status, StatusCode::OK, "reviewer login")?;
  let reviewer = text(api::data(&login), "token")?;
  let reviewer_id = api::id(&api::data(&login)["user_id"], "reviewer")?.parse::<i64>()?;
  reporter.remember_secret(&reviewer);
  let mut fixture = Fixture {
    client,
    config,
    owner: &scenario.owner_token,
    outsider: &scenario.outsider_token,
    project,
    project_path,
    reviewer,
    reviewer_id,
    milestone: String::new(),
  };
  fixture.ok(
    Method::POST,
    "/members",
    fixture.owner,
    Some(json!({"user_id":reviewer_id.to_string(),"role":"maintainer"})),
  )?;
  fixture.commit("main", "blackbox/base.txt")?;
  let milestone = fixture.ok(Method::POST, "/milestones", fixture.owner,
    Some(json!({"title":"Recovery release","description":"Real-image acceptance","due_date":"2030-12-31"})))?;
  fixture.milestone = api::id(&milestone["id"], "milestone")?;
  fixture.ok(
    Method::POST,
    "/labels",
    fixture.owner,
    Some(json!({"title":"recovery","color":"#428BCA","description":"Durability verification"})),
  )?;
  fixture.ok(Method::POST, "/merge-request-approval-rules", fixture.owner,
    Some(json!({"name":"Independent reviewer","target_branch":"main","approvals_required":1,"eligible_user_ids":[reviewer_id]})))?;
  Ok(fixture)
}
