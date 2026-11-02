//! Decodes a runner-produced `index.scip` into platform-shaped structures.
//!
//! The runner stores the indexer's output zstd-compressed; the protobuf payload
//! inside is decoded with the generated types from `proto/scip.proto`, then
//! converted to 1-based positions. Nothing here reads the repository or talks to
//! a database, so the server can rebuild a shard from the artifact alone.

use crate::model::{
  Decoded, DecodedDocument, DecodedOccurrence, DecodedSymbol, PositionEncoding, Span, SymbolKind,
  SymbolRole,
};
use crate::proto;
use crate::{Error, Result};
use prost::Message;
use std::io::Read;

/// Decompressed ceiling: a corrupt or hostile artifact must not exhaust memory.
pub const MAX_INDEX_BYTES: usize = 512 * 1024 * 1024;

/// Decodes a zstd-compressed `index.scip` produced for `commit`.
///
/// SCIP itself carries no commit, so the caller supplies the revision the index
/// was built at; it ends up in the shard manifest.
pub fn decode_index(bytes: &[u8], commit: &str) -> Result<Decoded> {
  let decoder = zstd::stream::read::Decoder::new(bytes).map_err(Error::Decompress)?;
  let mut payload = Vec::new();
  decoder
    .take(MAX_INDEX_BYTES as u64 + 1)
    .read_to_end(&mut payload)
    .map_err(Error::Decompress)?;
  if payload.len() > MAX_INDEX_BYTES {
    return Err(Error::TooLarge {
      limit: MAX_INDEX_BYTES,
    });
  }
  decode_scip(&payload, commit)
}

/// Decodes the uncompressed protobuf payload of an `Index` message.
pub fn decode_scip(payload: &[u8], commit: &str) -> Result<Decoded> {
  let index = proto::Index::decode(payload)?;
  let metadata = index.metadata.unwrap_or_default();
  let tool = metadata.tool_info.unwrap_or_default();
  let mut decoded = Decoded {
    commit: commit.to_string(),
    indexer: tool.name,
    indexer_version: tool.version,
    ..Decoded::default()
  };
  decoded.documents = index
    .documents
    .iter()
    .map(document)
    .collect::<Result<Vec<DecodedDocument>>>()?;
  Ok(decoded)
}

fn document(raw: &proto::Document) -> Result<DecodedDocument> {
  Ok(DecodedDocument {
    path: crate::assemble::normalize(&raw.relative_path)?,
    language: raw.language.clone(),
    position_encoding: PositionEncoding::from_code(raw.position_encoding),
    occurrences: raw.occurrences.iter().map(occurrence).collect(),
    symbols: raw.symbols.iter().map(symbol).collect(),
    text: raw.text.clone(),
  })
}

// The deprecated `range`/`enclosing_range` fields are still what most indexers
// emit, so both encodings are read here on purpose.
#[allow(deprecated)]
fn occurrence(raw: &proto::Occurrence) -> DecodedOccurrence {
  DecodedOccurrence {
    span: range(&raw.typed_range, &raw.range).unwrap_or_default(),
    enclosing: range_enclosing(raw),
    symbol: raw.symbol.clone(),
    roles: SymbolRole::from_bits(raw.symbol_roles as u32),
    syntax_kind: raw.syntax_kind,
  }
}

fn range(typed: &Option<proto::occurrence::TypedRange>, legacy: &[i32]) -> Option<Span> {
  if let Some(typed) = typed {
    return Some(match typed {
      proto::occurrence::TypedRange::SingleLineRange(range) => span(
        range.line,
        range.start_character,
        range.line,
        range.end_character,
      ),
      proto::occurrence::TypedRange::MultiLineRange(range) => span(
        range.start_line,
        range.start_character,
        range.end_line,
        range.end_character,
      ),
    });
  }
  legacy_span(legacy)
}

#[allow(deprecated)]
fn range_enclosing(raw: &proto::Occurrence) -> Option<Span> {
  if let Some(typed) = &raw.typed_enclosing_range {
    let converted = match typed {
      proto::occurrence::TypedEnclosingRange::SingleLineEnclosingRange(range) => span(
        range.line,
        range.start_character,
        range.line,
        range.end_character,
      ),
      proto::occurrence::TypedEnclosingRange::MultiLineEnclosingRange(range) => span(
        range.start_line,
        range.start_character,
        range.end_line,
        range.end_character,
      ),
    };
    return Some(converted);
  }
  legacy_span(&raw.enclosing_range)
}

/// The deprecated encoding is 3 or 4 ints; the typed one always wins.
fn legacy_span(range: &[i32]) -> Option<Span> {
  match range {
    [line, start, end] => Some(span(*line, *start, *line, *end)),
    [start_line, start, end_line, end] => Some(span(*start_line, *start, *end_line, *end)),
    _ => None,
  }
}

fn span(start_line: i32, start_column: i32, end_line: i32, end_column: i32) -> Span {
  Span::new(
    one_based(start_line),
    one_based(start_column),
    one_based(end_line),
    one_based(end_column),
  )
}

fn one_based(value: i32) -> u32 {
  u32::try_from(value).unwrap_or(0).saturating_add(1)
}

fn symbol(raw: &proto::SymbolInformation) -> DecodedSymbol {
  let documentation = raw
    .documentation
    .iter()
    .filter(|doc| !doc.is_empty())
    .cloned()
    .collect::<Vec<String>>()
    .join("\n\n");
  DecodedSymbol {
    symbol: raw.symbol.clone(),
    kind: SymbolKind::from_code(raw.kind),
    documentation,
    signature: raw
      .signature_documentation
      .as_ref()
      .map(|signature| signature.text.clone())
      .unwrap_or_default(),
  }
}
