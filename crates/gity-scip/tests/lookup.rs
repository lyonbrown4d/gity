//! Position lookups: binary search over one path's occurrence range, hover
//! payloads, and the difference between "absent" and "uncovered".

mod support;

use gity_scip::{
  Answer, Decoded, DecodedDocument, Hover, Location, PositionEncoding, Shard, Span, SymbolRole,
};

fn open_sample() -> (tempfile::TempDir, Shard) {
  let (root, dir) = support::root();
  Shard::build(&dir, &support::sample()).expect("build");
  let shard = Shard::open(&dir).expect("open").expect("shard present");
  (root, shard)
}

fn definition() -> Location {
  Location {
    path: "src/lib.rs".to_string(),
    line: 3,
    column: 8,
    end_line: 3,
    end_column: 11,
  }
}

#[test]
fn definition_at_follows_a_use_site_to_its_definition() {
  let (_root, shard) = open_sample();
  assert_eq!(
    shard.definition_at("src/lib.rs", 12, 6),
    Answer::Found(definition())
  );
}

#[test]
fn definition_at_resolves_a_cross_file_definition() {
  let (_root, shard) = open_sample();
  assert_eq!(
    shard.definition_at("src/main.rs", 5, 10),
    Answer::Found(definition())
  );
}

#[test]
fn definition_at_accepts_the_first_column_of_an_occurrence() {
  let (_root, shard) = open_sample();
  assert_eq!(
    shard.definition_at("src/lib.rs", 20, 4),
    Answer::Found(definition())
  );
  // A definition occurrence resolves to itself.
  assert_eq!(
    shard.definition_at("src/lib.rs", 3, 9),
    Answer::Found(definition())
  );
}

#[test]
fn definition_at_treats_spans_as_half_open() {
  let (_root, shard) = open_sample();
  // The `Foo` reference at line 20 covers columns 4..7, so 7 is outside.
  assert_eq!(
    shard.definition_at("src/lib.rs", 20, 6),
    Answer::Found(definition())
  );
  assert_eq!(shard.definition_at("src/lib.rs", 20, 7), Answer::Absent);
  assert_eq!(shard.definition_at("src/lib.rs", 4, 1), Answer::Absent);
}

#[test]
fn definition_at_reports_absent_inside_an_analysed_file() {
  let (_root, shard) = open_sample();
  assert_eq!(shard.definition_at("src/lib.rs", 30, 1), Answer::Absent);
  // Occurrences without a symbol only exist for highlighting.
  assert_eq!(shard.definition_at("src/lib.rs", 25, 2), Answer::Absent);
}

#[test]
fn definition_at_reports_uncovered_paths_explicitly() {
  let (_root, shard) = open_sample();
  assert_eq!(
    shard.definition_at(support::SKIPPED, 1, 1),
    Answer::Uncovered
  );
  assert_eq!(
    shard.definition_at("docs/unknown.rs", 1, 1),
    Answer::Uncovered
  );
}

#[test]
fn hover_returns_documentation_signature_and_scope() {
  let (_root, shard) = open_sample();
  let answer = shard.hover("src/lib.rs", 12, 6);
  let Answer::Found(hover) = answer else {
    panic!("expected hover, got {answer:?}");
  };
  assert_eq!(
    hover,
    Hover {
      symbol: support::FOO.to_string(),
      kind: gity_scip::SymbolKind::from_code(7),
      documentation: "The Foo type.".to_string(),
      signature: "struct Foo".to_string(),
      scope: support::BAR.to_string(),
    }
  );
  assert_eq!(hover.kind.name(), "Class");
}

#[test]
fn hover_on_a_top_level_definition_has_no_scope() {
  let (_root, shard) = open_sample();
  let Answer::Found(hover) = shard.hover("src/lib.rs", 3, 9) else {
    panic!("expected hover at the definition");
  };
  assert_eq!(hover.symbol, support::FOO);
  assert!(hover.scope.is_empty(), "scope {} not empty", hover.scope);
}

#[test]
fn hover_reports_absent_for_a_quiet_position() {
  let (_root, shard) = open_sample();
  assert_eq!(shard.hover("src/lib.rs", 30, 1), Answer::Absent);
  assert_eq!(shard.hover(support::SKIPPED, 1, 1), Answer::Uncovered);
}

#[test]
fn positions_are_scoped_to_their_own_path() {
  let (_root, shard) = open_sample();
  // Column 8 line 3 exists in `src/lib.rs` only.
  assert_eq!(shard.definition_at("src/main.rs", 3, 8), Answer::Absent);
}

#[test]
fn position_encoding_is_recorded_per_document() {
  let (_root, shard) = open_sample();
  assert_eq!(
    shard.position_encoding("src/lib.rs"),
    Some(PositionEncoding::Utf8)
  );
  assert_eq!(
    shard.position_encoding(support::SKIPPED),
    Some(PositionEncoding::Unspecified)
  );
  assert_eq!(shard.position_encoding("docs/unknown.rs"), None);
}

#[test]
fn lookups_hold_across_a_long_file() {
  let (_root, dir) = support::root();
  let mut decoded = Decoded::new(support::COMMIT);
  let mut occurrences = Vec::new();
  let mut symbols = Vec::new();
  for index in 0..400u32 {
    let name = format!("rust-analyzer cargo demo 0.1.0 item{index}().");
    let line = index * 3 + 1;
    occurrences.push(support::occurrence(
      line,
      1,
      line,
      6,
      &name,
      SymbolRole::DEFINITION,
    ));
    occurrences.push(support::occurrence(
      line + 1,
      2,
      line + 1,
      7,
      &name,
      SymbolRole::READ,
    ));
    symbols.push(support::symbol(&name, "", "", 0));
  }
  decoded.documents.push(DecodedDocument {
    path: "src/big.rs".to_string(),
    language: "rust".to_string(),
    position_encoding: PositionEncoding::Utf8,
    occurrences,
    symbols,
    text: String::new(),
  });
  Shard::build(&dir, &decoded).expect("build");
  let shard = Shard::open(&dir).expect("open").expect("shard present");
  let spot = shard.definition_at("src/big.rs", 300 * 3 + 2, 3);
  let Answer::Found(location) = spot else {
    panic!("expected a definition, got {spot:?}");
  };
  assert_eq!(
    location.span(),
    Span::new(300 * 3 + 1, 1, 300 * 3 + 1, 6),
    "the 300th reference must resolve to the 300th definition"
  );
}
