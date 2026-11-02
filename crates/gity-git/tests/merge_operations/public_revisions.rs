use super::*;

#[tokio::test]
async fn internal_receipts_are_not_public_revisions_or_branch_sources() {
  let (dir, git, input) = fixture().await;
  let applied = git.merge_operation("20.git", input.clone()).await.unwrap();
  let repo = git2::Repository::open_bare(dir.path().join("20.git")).unwrap();
  let marker = format!("refs/gity/merge-operations/{}", input.operation_key);
  let receipt = repo
    .find_reference(&marker)
    .unwrap()
    .target()
    .unwrap()
    .to_string();
  let hidden = [
    marker.clone(),
    marker.strip_prefix("refs/").unwrap().into(),
    format!("{marker}^{{commit}}"),
    receipt.clone(),
    receipt[..12].into(),
    format!("{}^0", marker.strip_prefix("refs/").unwrap()),
  ];
  for revision in hidden {
    assert!(
      git.resolve_revision("20.git", &revision, "").await.is_err(),
      "{revision}"
    );
    assert!(
      git
        .get_blob("20.git", &revision, "", "receipt.json")
        .await
        .is_err(),
      "{revision}"
    );
    assert!(
      git
        .create_branch("20.git", "leaked", &revision)
        .await
        .is_err(),
      "{revision}"
    );
    assert!(
      git.create_tag("20.git", "leaked", &revision).await.is_err(),
      "{revision}"
    );
  }
  assert!(git.spawn_archive("20.git", &receipt).await.is_err());
  for revision in ["main", "refs/heads/main", sha(&applied), "HEAD", "main^0"] {
    assert_eq!(
      git.resolve_revision("20.git", revision, "").await.unwrap(),
      sha(&applied)
    );
  }
  git.create_tag("20.git", "v1", "main").await.unwrap();
  assert_eq!(
    git.resolve_revision("20.git", "v1^{}", "").await.unwrap(),
    sha(&applied)
  );
  assert_eq!(
    git
      .resolve_revision("20.git", &input.target_sha, "")
      .await
      .unwrap(),
    input.target_sha
  );
  git
    .create_branch("20.git", "gity/ordinary", "main")
    .await
    .unwrap();
  assert_eq!(
    git
      .resolve_revision("20.git", "gity/ordinary", "")
      .await
      .unwrap(),
    sha(&applied)
  );
  assert_eq!(
    git.merge_operation_outcome("20.git", input).await.unwrap(),
    Some(applied)
  );
}

#[tokio::test]
async fn prepared_only_candidate_is_private_until_publicly_reachable() {
  let (dir, git, input) = fixture().await;
  let applied = git.merge_operation("20.git", input.clone()).await.unwrap();
  let repo = git2::Repository::open_bare(dir.path().join("20.git")).unwrap();
  repo
    .find_reference(&format!(
      "refs/gity/merge-operations/{}",
      input.operation_key
    ))
    .unwrap()
    .delete()
    .unwrap();
  repo
    .reference(
      "refs/heads/main",
      git2::Oid::from_str(&input.target_sha).unwrap(),
      true,
      "simulate prepare-only crash",
    )
    .unwrap();
  assert!(git
    .resolve_revision("20.git", sha(&applied), "")
    .await
    .is_err());
  assert!(git
    .create_branch("20.git", "unapplied", sha(&applied))
    .await
    .is_err());
  assert!(git
    .merge_operation_outcome("20.git", input.clone())
    .await
    .unwrap()
    .is_none());
  assert_eq!(git.merge_operation("20.git", input).await.unwrap(), applied);
  assert_eq!(
    git
      .resolve_revision("20.git", sha(&applied), "")
      .await
      .unwrap(),
    sha(&applied)
  );
}
