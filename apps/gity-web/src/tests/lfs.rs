use super::bytes;
#[test]
fn byte_sizes_match_original_units_and_rounding() {
  assert_eq!(bytes(0), "0 B");
  assert_eq!(bytes(1023), "1023 B");
  assert_eq!(bytes(1024), "1.0 KiB");
  assert_eq!(bytes(1536), "1.5 KiB");
  assert_eq!(bytes(1024 * 1024), "1.0 MiB");
  assert_eq!(bytes(3 * 1024 * 1024), "3.0 MiB");
}
