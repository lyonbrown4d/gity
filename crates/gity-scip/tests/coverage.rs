//! Coverage answers: an empty shard, a listed-but-unanalysed path, and the
//! definite empty result a precise path may still return.

mod support;

use gity_scip::{Answer, Coverage, Decoded, Shard};

fn open_sample() -> (tempfile::TempDir, Shard) {
  let (root, dir) = support::root();
  Shard::build(&dir, &support::sample()).expect("build");
  let shard = Shard::open(&dir).expect("open").expect("shard present");
  (root, shard)
}

#[test]
fn coverage_separates_precise_listed_and_unknown_paths() {
  let (_root, shard) = open_sample();
  assert_eq!(shard.coverage("src/lib.rs"), Coverage::Precise);
  assert_eq!(shard.coverage("src/main.rs"), Coverage::Precise);
  assert_eq!(shard.coverage(support::SKIPPED), Coverage::Listed);
  assert_eq!(shard.coverage("docs/unknown.rs"), Coverage::Unknown);
  assert!(shard.coverage("src/lib.rs").is_precise());
  assert!(!shard.coverage(support::SKIPPED).is_precise());
}

#[test]
fn a_listed_path_never_answers_from_the_precise_layer() {
  let (_root, shard) = open_sample();
  assert_eq!(shard.coverage(support::SKIPPED), Coverage::Listed);
  assert_eq!(
    shard.definition_at(support::SKIPPED, 1, 1),
    Answer::Uncovered
  );
  assert_eq!(shard.hover(support::SKIPPED, 1, 1), Answer::Uncovered);
  assert_eq!(
    shard.position_encoding(support::SKIPPED),
    Some(gity_scip::PositionEncoding::Unspecified)
  );
}

#[test]
fn an_empty_shard_reports_every_path_uncovered() {
  let (_root, dir) = support::root();
  let manifest = Shard::build(&dir, &Decoded::default()).expect("build empty shard");
  assert_eq!(manifest.counts.paths, 0);
  assert_eq!(manifest.coverage.total_paths, 0);
  let shard = Shard::open(&dir).expect("open").expect("shard present");
  assert_eq!(shard.coverage("src/lib.rs"), Coverage::Unknown);
  assert_eq!(shard.definition_at("src/lib.rs", 1, 1), Answer::Uncovered);
  assert_eq!(shard.hover("src/lib.rs", 1, 1), Answer::Uncovered);
  assert!(shard.references(support::FOO).is_empty());
}

#[test]
fn a_precise_path_without_symbols_answers_absent() {
  let (_root, dir) = support::root();
  let mut decoded = Decoded::new(support::COMMIT);
  decoded
    .documents
    .push(support::document("src/empty.rs", Vec::new(), Vec::new()));
  Shard::build(&dir, &decoded).expect("build");
  let shard = Shard::open(&dir).expect("open").expect("shard present");
  assert_eq!(shard.coverage("src/empty.rs"), Coverage::Precise);
  // Analysed and empty: a definite answer, unlike `Uncovered`.
  assert_eq!(shard.definition_at("src/empty.rs", 1, 1), Answer::Absent);
  assert_eq!(shard.hover("src/empty.rs", 1, 1), Answer::Absent);
}

#[test]
fn an_analysed_path_stays_precise_even_if_the_runner_listed_it_as_skipped() {
  let (_root, dir) = support::root();
  let mut decoded = support::sample();
  decoded.uncovered_paths.push("src/lib.rs".to_string());
  let manifest = Shard::build(&dir, &decoded).expect("build");
  assert_eq!(manifest.coverage.listed_paths, 1);
  let shard = Shard::open(&dir).expect("open").expect("shard present");
  assert_eq!(shard.coverage("src/lib.rs"), Coverage::Precise);
}

#[test]
fn a_stale_shard_does_not_claim_paths_it_never_indexed() {
  let (_root, dir) = support::root();
  Shard::build(&dir, &support::sample()).expect("build");
  let shard = Shard::open(&dir).expect("open").expect("shard present");
  // Paths added to the repository after the index was built must degrade.
  assert_eq!(shard.coverage("src/added_later.rs"), Coverage::Unknown);
  assert_eq!(
    shard.definition_at("src/added_later.rs", 1, 1),
    Answer::Uncovered
  );
}
