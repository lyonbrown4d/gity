//! Column conversion for documents indexed in UTF-16 or UTF-32 units.

mod support;

use gity_scip::{Answer, PositionEncoding, Shard};

fn open_utf16() -> (tempfile::TempDir, Shard) {
  let (root, dir) = support::root();
  Shard::build(&dir, &support::utf16_sample()).expect("build");
  let shard = Shard::open(&dir).expect("open").expect("shard present");
  (root, shard)
}

#[test]
fn manifest_records_the_column_unit() {
  let (_root, shard) = open_utf16();
  assert_eq!(shard.manifest().position_encoding, "utf16");
  assert_eq!(
    shard.position_encoding("src/emoji.rs"),
    Some(PositionEncoding::Utf16)
  );
}

#[test]
fn normalized_position_counts_utf16_units_through_an_emoji_and_cjk() {
  let (_root, shard) = open_utf16();
  // `let 🚀名 = Foo;`: the cursor before `Foo` is UTF-8 byte column 15, which is
  // UTF-16 column 11 (4 ASCII + 2 for the emoji + 1 for 名 + 3 ASCII, plus 1).
  assert_eq!(shard.normalized_position("src/emoji.rs", 1, 15), Some(11));
  // A column inside the emoji rounds down to its start: prefix `let ` is four
  // UTF-16 units, so the emoji itself starts at column 5.
  assert_eq!(shard.normalized_position("src/emoji.rs", 1, 6), Some(5));
  // Beyond the line clamps to the line end: 18 bytes, 14 UTF-16 units.
  assert_eq!(shard.normalized_position("src/emoji.rs", 1, 999), Some(15));
}

#[test]
fn normalized_position_feeds_definition_at() {
  let (_root, shard) = open_utf16();
  let column = shard
    .normalized_position("src/emoji.rs", 1, 15)
    .expect("text is stored");
  assert_eq!(
    shard.definition_at("src/emoji.rs", 1, column),
    Answer::Found(gity_scip::Location {
      path: "src/emoji.rs".to_string(),
      line: 1,
      column: 11,
      end_line: 1,
      end_column: 14,
      external: false,
    })
  );
}

#[test]
fn utf8_documents_need_no_conversion() {
  let (root, dir) = support::root();
  Shard::build(&dir, &support::sample()).expect("build");
  let shard = Shard::open(&dir).expect("open").expect("shard present");
  assert_eq!(
    shard.normalized_position("src/lib.rs", 20, 4),
    Some(4),
    "a UTF-8 byte column is already the shard's unit"
  );
  assert_eq!(
    shard.normalized_position("docs/unknown.rs", 1, 1),
    None,
    "unknown paths cannot be converted"
  );
  drop(root);
}

#[test]
fn a_utf16_document_without_text_cannot_be_converted() {
  let (_root, shard) = open_utf16();
  assert_eq!(
    shard.position_encoding("src/no_text.rs"),
    Some(PositionEncoding::Utf16)
  );
  assert_eq!(shard.normalized_position("src/no_text.rs", 1, 5), None);
  assert_eq!(shard.source_text("src/no_text.rs"), None);
}

#[test]
fn stored_source_text_is_available_for_tooltips() {
  let (_root, shard) = open_utf16();
  assert_eq!(shard.source_text("src/emoji.rs"), Some(support::UTF16_TEXT));
}

#[test]
fn column_conversion_is_available_without_a_shard() {
  let line = "let 🚀名 = Foo;";
  assert_eq!(PositionEncoding::Utf8.column(line, 15), 15);
  assert_eq!(PositionEncoding::Utf16.column(line, 15), 11);
  assert_eq!(PositionEncoding::Utf32.column(line, 15), 10);
  assert_eq!(
    PositionEncoding::Unspecified.column(line, 15),
    15,
    "an unspecified document is assumed to use bytes"
  );
  // Columns inside a multi-byte character round down instead of panicking.
  assert_eq!(PositionEncoding::Utf8.column(line, 6), 5);
}

#[test]
fn mixed_encodings_are_reported_as_mixed() {
  let (_root, dir) = support::root();
  let mut decoded = support::utf16_sample();
  decoded
    .documents
    .push(support::document("src/plain.rs", Vec::new(), Vec::new()));
  let manifest = Shard::build(&dir, &decoded).expect("build");
  assert_eq!(manifest.position_encoding, "mixed");
  assert_eq!(manifest.counts.text_bytes, support::UTF16_TEXT.len());
}
