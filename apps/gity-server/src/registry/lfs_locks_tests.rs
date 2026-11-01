use super::*;
#[test]
fn lock_paths_reject_traversal_and_empty_segments() {
    assert_eq!(path("dir\\file.bin").unwrap(), "dir/file.bin");
    assert!(path("dir/../file").is_err());
    assert!(path("dir//file").is_err());
    assert!(path("").is_err());
}
