use super::*;
#[test]
fn slugs_normalize_and_reject_empty_paths() {
    assert_eq!(slug(" Getting Started ").unwrap(), "getting-started");
    assert_eq!(slug("Café Guide").unwrap(), "cafe-guide");
    assert!(slug("../").is_err());
    assert!(slug(&"x".repeat(161)).is_err());
}
