use crate::proto;
use crate::{Error, SymbolKind, SymbolRole, decode_index, decode_scip};
use prost::Message;

const FOO: &str = "rust-analyzer cargo demo 0.1.0 src/Foo#";

#[allow(deprecated)]
fn sample_index() -> proto::Index {
  proto::Index {
    metadata: Some(proto::Metadata {
      version: 0,
      tool_info: Some(proto::ToolInfo {
        name: "rust-analyzer".into(),
        version: "2026-10-01".into(),
        arguments: Vec::new(),
      }),
      project_root: "file:///repo".into(),
      text_document_encoding: 1,
    }),
    documents: vec![proto::Document {
      language: "rust".into(),
      relative_path: "src/lib.rs".into(),
      occurrences: vec![
        proto::Occurrence {
          range: vec![2, 4, 7],
          symbol: FOO.into(),
          symbol_roles: SymbolRole::DEFINITION.bits() as i32,
          ..proto::Occurrence::default()
        },
        proto::Occurrence {
          typed_range: Some(proto::occurrence::TypedRange::SingleLineRange(
            proto::SingleLineRange {
              line: 19,
              start_character: 4,
              end_character: 8,
            },
          )),
          symbol: FOO.into(),
          symbol_roles: SymbolRole::READ.bits() as i32,
          syntax_kind: 1,
          ..proto::Occurrence::default()
        },
      ],
      symbols: vec![proto::SymbolInformation {
        symbol: FOO.into(),
        documentation: vec!["The Foo type.".into()],
        signature_documentation: Some(proto::Signature {
          language: "rust".into(),
          text: "struct Foo".into(),
          occurrences: Vec::new(),
        }),
        kind: 7,
        ..proto::SymbolInformation::default()
      }],
      text: String::new(),
      position_encoding: 1,
    }],
    external_symbols: Vec::new(),
  }
}

fn encode(index: &proto::Index) -> Vec<u8> {
  let mut payload = Vec::new();
  index.encode(&mut payload).expect("encode protobuf");
  payload
}

#[test]
fn decode_scip_shifts_positions_to_one_based() {
  let decoded = decode_scip(&encode(&sample_index()), "0123456789abcdef").expect("decode");
  assert_eq!(decoded.commit, "0123456789abcdef");
  assert_eq!(decoded.indexer, "rust-analyzer");
  assert_eq!(decoded.indexer_version, "2026-10-01");
  assert_eq!(decoded.documents.len(), 1);
  let document = &decoded.documents[0];
  assert_eq!(document.path, "src/lib.rs");
  assert_eq!(document.language, "rust");
  assert_eq!(document.position_encoding, crate::PositionEncoding::Utf8);
  assert_eq!(document.occurrences[0].span, crate::Span::new(3, 5, 3, 8));
  assert!(document.occurrences[0].roles.is_definition());
  assert_eq!(document.occurrences[1].span, crate::Span::new(20, 5, 20, 9));
  assert_eq!(document.occurrences[1].roles, SymbolRole::READ);
  assert_eq!(document.occurrences[1].syntax_kind, 1);
  assert_eq!(document.symbols.len(), 1);
  let symbol = &document.symbols[0];
  assert_eq!(symbol.documentation, "The Foo type.");
  assert_eq!(symbol.signature, "struct Foo");
  assert_eq!(symbol.kind, SymbolKind::from_code(7));
  assert_eq!(symbol.kind.name(), "Class");
}

#[test]
fn decode_index_unwraps_the_zstd_artifact() {
  let payload = encode(&sample_index());
  let compressed = zstd::encode_all(payload.as_slice(), 3).expect("compress");
  let decoded = decode_index(&compressed, "0123456789abcdef").expect("decode");
  assert_eq!(decoded.documents[0].occurrences.len(), 2);
}

#[test]
fn decode_index_rejects_a_non_zstd_artifact() {
  let error = decode_index(b"not zstd at all", "0123456789abcdef").expect_err("garbage");
  assert!(matches!(error, Error::Decompress(_)), "{error}");
}

#[test]
fn decode_scip_rejects_a_truncated_payload() {
  let payload = encode(&sample_index());
  let error =
    decode_scip(&payload[..payload.len() / 2], "0123456789abcdef").expect_err("truncated");
  assert!(matches!(error, Error::Proto(_)), "{error}");
}

#[test]
fn decode_scip_rejects_a_document_without_a_path() {
  for bad in ["", "/", "src/lib.rs "] {
    let mut index = sample_index();
    index.documents[0].relative_path = bad.into();
    let error = decode_scip(&encode(&index), "0123456789abcdef").expect_err("bad path");
    assert!(matches!(error, Error::Path(_)), "{bad:?}: {error}");
  }
}
