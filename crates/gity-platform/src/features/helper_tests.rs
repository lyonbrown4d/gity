use super::*;
#[tokio::test]
async fn project_lookup_and_collection_replacement_are_isolated_and_atomic() {
  let (_dir, state) = super::test_support::fixture().await;
  for (id, project) in [(100, 20), (200, 30)] {
    state.db.insert("project_issues",fields(json!({"id":id,"project_id":project,"iid":1,"author_user_id":1,"title":"Scoped","state":"opened"}))).await.unwrap();
  }
  assert_eq!(state.collaboration.issue(30, 1).await.unwrap().id, 200);
  assert!(state.collaboration.issue(30, 100).await.is_err());
  state
    .collaboration
    .replace_assignees(20, 1, vec![1])
    .await
    .unwrap();
  // Failure after deleting the old collection and inserting the first duplicate
  // must roll the entire replacement back.
  assert!(
    state
      .collaboration
      .replace_assignees(20, 1, vec![2, 2])
      .await
      .is_err()
  );
  let rows = state.collaboration.issue_assignees(20, 1).await.unwrap();
  assert_eq!(rows.len(), 1);
  assert_eq!(rows[0].user_id, 1);
  assert!(
    state
      .collaboration
      .issue_assignees(30, 1)
      .await
      .unwrap()
      .is_empty()
  );
}
#[test]
fn issue_wire_contract_uses_project_local_iid() {
  let view = externalize(issues::view(
    json!({"id":99,"iid":3,"project_id":4,"author_user_id":5,"title":"Issue","state":"opened"}),
  ));
  assert_eq!(view["id"], "3");
  assert_eq!(view["repository_id"], "4");
  assert_eq!(view["number"], 3);
  assert_eq!(view["status"], "open");
  assert_eq!(view["author_user_id"], "5");
}
