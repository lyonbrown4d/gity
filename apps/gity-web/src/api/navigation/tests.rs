use super::safe_after_login_redirect;
#[test]
fn expired_login_preserves_only_the_local_consent_request() {
  let id = "019a4260-40bf-7000-8000-000000000001";
  let target = format!("/oauth/consent?request_id={id}");
  assert_eq!(safe_after_login_redirect(&target), Some(target.clone()));
  assert_eq!(
    safe_after_login_redirect(&format!("{target}&next=https://evil.example")),
    Some(target)
  );
  for target in [
    "https://evil.example/oauth/consent",
    "//evil.example/oauth/consent",
    "/oauth/consent?request_id=broken",
    "/other?request_id=019a4260-40bf-7000-8000-000000000001",
  ] {
    assert!(safe_after_login_redirect(target).is_none());
  }
}
