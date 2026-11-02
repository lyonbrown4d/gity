use super::*;
pub(super) use gity_projects::OrganizationMemberChange as Change;
pub(super) async fn mutate(
  s: &AppState,
  headers: &HeaderMap,
  organization: i64,
  user: i64,
  change: Change<'_>,
) -> Reply {
  let actor = auth::principal(s, headers).await?;
  let member = s
    .projects
    .change_organization_member(&actor, organization, user, change)
    .await?;
  let Some(member) = member else {
    return Ok(ok(json!({"ok":true})));
  };
  let user_model = s
    .projects
    .users(vec![user])
    .await?
    .into_iter()
    .next()
    .ok_or_else(|| AppError::not_found("user not found"))?;
  Ok(ok(
    json!({"id":member.id,"organization_id":member.organization_id,"user_id":member.user_id,"role":member.role,"created_at":member.created_at,"updated_at":member.updated_at,"username":user_model.username,"display_name":user_model.display_name,"email":user_model.email}),
  ))
}
