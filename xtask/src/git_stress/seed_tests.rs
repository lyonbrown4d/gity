use super::random_file;
use sha2::{Digest, Sha256};
#[test]
fn configured_seed_size_is_exact_and_files_do_not_reuse_compressible_filler() {
  let directory = tempfile::tempdir().unwrap();
  let first = directory.path().join("first.bin");
  let second = directory.path().join("second.bin");
  random_file(&first, 1024 * 1024).unwrap();
  random_file(&second, 1024 * 1024).unwrap();
  let first = std::fs::read(first).unwrap();
  let second = std::fs::read(second).unwrap();
  assert_eq!(first.len(), 1024 * 1024);
  assert_ne!(Sha256::digest(&first), Sha256::digest(&second));
  assert_ne!(&first[..128 * 1024], &first[128 * 1024..256 * 1024]);
}
