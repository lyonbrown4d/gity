use super::{ensure_allowed, same_origin};
use http::Uri;
use std::{collections::BTreeSet, io};

#[test]
fn allows_only_exact_https_registry_and_auth_realm_hosts() {
  let allowed = BTreeSet::from(["registry.example".to_owned(), "auth.example".to_owned()]);
  ensure_allowed(
    &"https://registry.example/v2/team/repo/manifests/latest"
      .parse::<Uri>()
      .unwrap(),
    &allowed,
  )
  .unwrap();
  ensure_allowed(
    &"https://auth.example/token?service=registry.example"
      .parse::<Uri>()
      .unwrap(),
    &allowed,
  )
  .unwrap();
}

#[test]
fn rejects_unlisted_bearer_realms_and_redirect_destinations() {
  let allowed = BTreeSet::from(["registry.example".to_owned()]);
  let malicious_realm: Uri = "https://attacker.example/token".parse().unwrap();
  let external_redirect: Uri = "https://cdn.attacker.example/blob".parse().unwrap();
  assert!(ensure_allowed(&malicious_realm, &allowed).is_err());
  assert!(ensure_allowed(&external_redirect, &allowed).is_err());
}

#[test]
fn rejects_http_downgrade_even_if_registry_name_is_allowlisted() {
  let allowed = BTreeSet::from(["registry.example".to_owned()]);
  let downgrade: Uri = "http://registry.example/v2/team/repo/manifests/latest"
    .parse()
    .unwrap();
  assert_eq!(
    ensure_allowed(&downgrade, &allowed).unwrap_err().kind(),
    io::ErrorKind::Other
  );
}

#[test]
fn cross_origin_redirect_drops_authorization_scope() {
  let registry: Uri = "https://registry.example/v2/repo/manifests/tag"
    .parse()
    .unwrap();
  let auth_realm: Uri = "https://auth.example/token".parse().unwrap();
  let redirected_path: Uri = "https://registry.example/v2/repo/manifests/tag?x=1"
    .parse()
    .unwrap();
  assert!(!same_origin(&registry, &auth_realm));
  assert!(same_origin(&registry, &redirected_path));
}
