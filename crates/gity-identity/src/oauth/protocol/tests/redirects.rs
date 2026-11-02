use crate::oauth::protocol::{OAuthError, validate_redirect};

#[test]
fn callback_validation_requires_https_or_loopback_http() {
  assert!(validate_redirect("https://client.example/callback").is_ok());
  assert!(validate_redirect("http://127.0.0.1:43123/callback").is_ok());
  assert!(validate_redirect("http://127.0.0.2:43123/callback").is_ok());
  assert!(validate_redirect("http://localhost/callback").is_ok());
  assert!(matches!(
    validate_redirect("http://client.example/callback"),
    Err(OAuthError::Invalid)
  ));
  assert!(matches!(
    validate_redirect("https://user:pass@client.example/callback"),
    Err(OAuthError::Invalid)
  ));
  assert!(matches!(
    validate_redirect("https://client.example/callback#fragment"),
    Err(OAuthError::Invalid)
  ));
}
