use gity_code_intel::{Archive, Language, decode, encode};

fn sample() -> Archive {
  let file = Language::Rust
    .parse("src/lib.rs", "pub fn run() {}\n")
    .expect("parse");
  Archive::new("0123456789abcdef", vec![file])
}

#[test]
fn archive_round_trips_through_msgpack_and_zstd() {
  let archive = sample();
  let bytes = encode(&archive).expect("encode");
  let decoded = decode(&bytes).expect("decode");
  assert_eq!(decoded, archive);
  assert_eq!(decoded.definition_count(), 1);
  assert_eq!(decoded.commit, "0123456789abcdef");
}

#[test]
fn archive_rejects_unknown_versions() {
  let mut archive = sample();
  archive.version = 99;
  let bytes = encode(&archive).expect("encode");
  let error = decode(&bytes).expect_err("version mismatch");
  assert!(error.to_string().contains("not supported"), "{error}");
}

#[test]
fn archive_is_smaller_than_raw_source() {
  let archive = sample();
  let bytes = encode(&archive).expect("encode");
  assert!(bytes.len() < 4096, "unexpected size {}", bytes.len());
}
