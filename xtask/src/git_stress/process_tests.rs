use super::{read_bounded, safe_remove};
use std::{io::Cursor, path::Path};

#[test]
fn diagnostic_reader_caps_retained_output_but_drains_input() {
    let input = vec![b'x'; 128 * 1024];
    let output = read_bounded(Cursor::new(input), 1024);
    assert!(output.starts_with(&"x".repeat(1024)));
    assert!(output.ends_with("[diagnostic output truncated]"));
    assert!(output.len() < 1100);
}

#[test]
fn cleanup_refuses_to_remove_its_root() {
    let root = std::env::temp_dir();
    assert!(safe_remove(&root, Path::new(&root)).is_err());
}
