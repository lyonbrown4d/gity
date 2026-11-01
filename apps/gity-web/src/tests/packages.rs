use super::*;
fn input() -> UploadInput {
    UploadInput {
        kind: "generic".into(),
        name: " demo ".into(),
        version: " 1.0 ".into(),
        filename: " artifact.txt ".into(),
        path: String::new(),
        content_type: " text/plain ".into(),
        inline: "你好 Rust".into(),
        encoded: String::new(),
    }
}
#[test]
fn upload_preserves_utf8_binary_and_normalizes_coordinates() {
    let value = input().body().unwrap();
    assert_eq!(value["name"], "demo");
    assert_eq!(value["version"], "1.0");
    assert_eq!(value["file_path"], "artifact.txt");
    assert_eq!(
        STANDARD
            .decode(value["content_base64"].as_str().unwrap())
            .unwrap(),
        "你好 Rust".as_bytes()
    );
    let mut file = input();
    file.encoded = STANDARD.encode([0, 255, 128]);
    file.path = " dist/bin ".into();
    let value = file.body().unwrap();
    assert_eq!(value["file_path"], "dist/bin");
    assert_eq!(
        STANDARD
            .decode(value["content_base64"].as_str().unwrap())
            .unwrap(),
        [0, 255, 128]
    );
}
#[test]
fn upload_requires_content_and_coordinates_and_counts_nested_versions() {
    let mut missing = input();
    missing.inline.clear();
    assert!(missing.body().is_err());
    let mut missing = input();
    missing.name = " ".into();
    assert!(missing.body().is_err());
    let detail = json!({"versions":[{"files":[{"id":"1"},{"id":"2"}]},{"files":[{"id":"3"}]}]});
    assert_eq!(file_count(&detail), 3);
    assert_eq!(bytes(1023), "1023 B");
    assert_eq!(bytes(1024), "1.0 KiB");
    assert_eq!(bytes(1048576), "1.0 MiB");
}
