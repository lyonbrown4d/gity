use super::*;
#[test]
fn optional_token_dates_use_null_not_legacy_zero_dates() {
  assert!(active(&json!({"expires_at":null,"revoked_at":null})));
  assert!(active(
    &json!({"expires_at":"2100-01-01T00:00:00Z","revoked_at":null})
  ));
  assert!(!active(
    &json!({"expires_at":"0001-01-01T00:00:00Z","revoked_at":null})
  ));
  assert!(!active(
    &json!({"expires_at":null,"revoked_at":"0001-01-01T00:00:00Z"})
  ));
  assert!(!active(&json!({"expires_at":"","revoked_at":null})));
}
