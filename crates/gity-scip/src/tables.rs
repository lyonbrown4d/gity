//! Record layout and (de)serialisation for the six shard tables.
//!
//! Every table is fixed-width records plus a pool, so opening a shard is a few
//! bulk reads and queries never parse text. Layouts, all little-endian:
//!
//! ```text
//! paths.bin     GSP1 | count | (text_off, text_len, occ_first, occ_len, encoding) | path pool
//! occ.bin       GSO1 | count | (start_line, start_col, end_line, end_col, symbol, roles, enclosing)
//! symbols.bin   GSS1 | count | (text_off, text_len, kind, doc_off, doc_len, sig_off, sig_len) | pool
//! external.bin  GSX1 | count | same records as symbols.bin | symbol pool
//! texts.bin     GST1 | count | (text_off, text_len) per path | text pool
//! docs.bin      GSD1 | length | documentation and signature bytes, flat
//! refs.bin      GSR1 | symbols | entries | CSR offsets (symbols + 1) | global occurrence IDs
//! coverage.bin  GSC1 | paths | bitset, one bit per path, MSB-first
//! ```
//!
//! Offsets are 0-based indices into the corresponding pool. `NONE` marks an
//! absent occurrence symbol or enclosing symbol.

use crate::Result;
use crate::format::{Reader, Writer};

pub(crate) const TAG_PATHS: [u8; 4] = *b"GSP1";
pub(crate) const TAG_OCC: [u8; 4] = *b"GSO1";
pub(crate) const TAG_SYMBOLS: [u8; 4] = *b"GSS1";
pub(crate) const TAG_DOCS: [u8; 4] = *b"GSD1";
pub(crate) const TAG_REFS: [u8; 4] = *b"GSR1";
pub(crate) const TAG_COVERAGE: [u8; 4] = *b"GSC1";
pub(crate) const TAG_EXTERNAL: [u8; 4] = *b"GSX1";
pub(crate) const TAG_TEXTS: [u8; 4] = *b"GST1";

/// Sentinel for "no symbol" in occurrence records.
pub(crate) const NONE: u32 = u32::MAX;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PathRecord {
  pub text_off: u32,
  pub text_len: u32,
  pub occ_first: u32,
  pub occ_len: u32,
  pub encoding: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct OccRecord {
  pub start_line: u32,
  pub start_column: u32,
  pub end_line: u32,
  pub end_column: u32,
  pub symbol: u32,
  pub roles: u32,
  pub enclosing: u32,
}

impl OccRecord {
  pub(crate) fn covers(&self, line: u32, column: u32) -> bool {
    let start = (self.start_line, self.start_column);
    let end = (self.end_line, self.end_column);
    (line, column) >= start && (line, column) < end
  }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SymbolRecord {
  pub text_off: u32,
  pub text_len: u32,
  pub kind: u32,
  pub doc_off: u32,
  pub doc_len: u32,
  pub signature_off: u32,
  pub signature_len: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextRecord {
  pub text_off: u32,
  pub text_len: u32,
}

pub(crate) fn encode_paths(records: &[PathRecord], pool: &[u8]) -> Vec<u8> {
  let mut writer = Writer::new(TAG_PATHS, records.len() as u32);
  for record in records {
    writer.u32(record.text_off);
    writer.u32(record.text_len);
    writer.u32(record.occ_first);
    writer.u32(record.occ_len);
    writer.u32(record.encoding);
  }
  writer.bytes(pool);
  writer.into_bytes()
}

pub(crate) fn decode_paths(bytes: &[u8]) -> Result<(Vec<PathRecord>, Vec<u8>)> {
  let mut reader = Reader::new(bytes, TAG_PATHS, "paths.bin")?;
  let count = reader.u32()? as usize;
  let mut records = Vec::with_capacity(count.min(1 << 20));
  for _ in 0..count {
    records.push(PathRecord {
      text_off: reader.u32()?,
      text_len: reader.u32()?,
      occ_first: reader.u32()?,
      occ_len: reader.u32()?,
      encoding: reader.u32()?,
    });
  }
  Ok((records, reader.rest().to_vec()))
}

pub(crate) fn encode_occ(records: &[OccRecord]) -> Vec<u8> {
  let mut writer = Writer::new(TAG_OCC, records.len() as u32);
  for record in records {
    writer.u32(record.start_line);
    writer.u32(record.start_column);
    writer.u32(record.end_line);
    writer.u32(record.end_column);
    writer.u32(record.symbol);
    writer.u32(record.roles);
    writer.u32(record.enclosing);
  }
  writer.into_bytes()
}

pub(crate) fn decode_occ(bytes: &[u8]) -> Result<Vec<OccRecord>> {
  let mut reader = Reader::new(bytes, TAG_OCC, "occ.bin")?;
  let count = reader.u32()? as usize;
  let mut records = Vec::with_capacity(count.min(1 << 22));
  for _ in 0..count {
    records.push(OccRecord {
      start_line: reader.u32()?,
      start_column: reader.u32()?,
      end_line: reader.u32()?,
      end_column: reader.u32()?,
      symbol: reader.u32()?,
      roles: reader.u32()?,
      enclosing: reader.u32()?,
    });
  }
  Ok(records)
}

pub(crate) fn encode_symbols(records: &[SymbolRecord], pool: &[u8]) -> Vec<u8> {
  encode_symbol_records(TAG_SYMBOLS, records, pool)
}

pub(crate) fn decode_symbols(bytes: &[u8]) -> Result<(Vec<SymbolRecord>, Vec<u8>)> {
  decode_symbol_records(bytes, TAG_SYMBOLS, "symbols.bin")
}

/// External symbols share the record layout but not the table: they have no
/// occurrence, so symbol IDs and the CSR stay untouched by them.
pub(crate) fn encode_external(records: &[SymbolRecord], pool: &[u8]) -> Vec<u8> {
  encode_symbol_records(TAG_EXTERNAL, records, pool)
}

pub(crate) fn decode_external(bytes: &[u8]) -> Result<(Vec<SymbolRecord>, Vec<u8>)> {
  decode_symbol_records(bytes, TAG_EXTERNAL, "external.bin")
}

fn encode_symbol_records(tag: [u8; 4], records: &[SymbolRecord], pool: &[u8]) -> Vec<u8> {
  let mut writer = Writer::new(tag, records.len() as u32);
  for record in records {
    writer.u32(record.text_off);
    writer.u32(record.text_len);
    writer.u32(record.kind);
    writer.u32(record.doc_off);
    writer.u32(record.doc_len);
    writer.u32(record.signature_off);
    writer.u32(record.signature_len);
  }
  writer.bytes(pool);
  writer.into_bytes()
}

fn decode_symbol_records(
  bytes: &[u8],
  tag: [u8; 4],
  table: &'static str,
) -> Result<(Vec<SymbolRecord>, Vec<u8>)> {
  let mut reader = Reader::new(bytes, tag, table)?;
  let count = reader.u32()? as usize;
  let mut records = Vec::with_capacity(count.min(1 << 20));
  for _ in 0..count {
    records.push(SymbolRecord {
      text_off: reader.u32()?,
      text_len: reader.u32()?,
      kind: reader.u32()?,
      doc_off: reader.u32()?,
      doc_len: reader.u32()?,
      signature_off: reader.u32()?,
      signature_len: reader.u32()?,
    });
  }
  Ok((records, reader.rest().to_vec()))
}

/// One record per path, in path-ID order, so text is found without a search.
pub(crate) fn encode_texts(records: &[TextRecord], pool: &[u8]) -> Vec<u8> {
  let mut writer = Writer::new(TAG_TEXTS, records.len() as u32);
  for record in records {
    writer.u32(record.text_off);
    writer.u32(record.text_len);
  }
  writer.bytes(pool);
  writer.into_bytes()
}

pub(crate) fn decode_texts(bytes: &[u8]) -> Result<(Vec<TextRecord>, Vec<u8>)> {
  let mut reader = Reader::new(bytes, TAG_TEXTS, "texts.bin")?;
  let count = reader.u32()? as usize;
  let mut records = Vec::with_capacity(count.min(1 << 20));
  for _ in 0..count {
    records.push(TextRecord {
      text_off: reader.u32()?,
      text_len: reader.u32()?,
    });
  }
  Ok((records, reader.rest().to_vec()))
}

pub(crate) fn encode_docs(blob: &[u8]) -> Vec<u8> {
  let mut writer = Writer::new(TAG_DOCS, blob.len() as u32);
  writer.bytes(blob);
  writer.into_bytes()
}

pub(crate) fn decode_docs(bytes: &[u8]) -> Result<Vec<u8>> {
  let mut reader = Reader::new(bytes, TAG_DOCS, "docs.bin")?;
  let length = reader.u32()? as usize;
  Ok(reader.slice(length)?.to_vec())
}

pub(crate) fn encode_refs(offsets: &[u32], entries: &[u32]) -> Vec<u8> {
  let symbols = offsets.len().saturating_sub(1) as u32;
  let mut writer = Writer::new(TAG_REFS, symbols);
  writer.u32(entries.len() as u32);
  for offset in offsets {
    writer.u32(*offset);
  }
  for entry in entries {
    writer.u32(*entry);
  }
  writer.into_bytes()
}

pub(crate) fn decode_refs(bytes: &[u8]) -> Result<(Vec<u32>, Vec<u32>)> {
  let mut reader = Reader::new(bytes, TAG_REFS, "refs.bin")?;
  let symbols = reader.u32()? as usize;
  let entries = reader.u32()? as usize;
  let mut offsets = Vec::with_capacity(symbols.saturating_add(1).min(1 << 20));
  for _ in 0..=symbols {
    offsets.push(reader.u32()?);
  }
  let mut ids = Vec::with_capacity(entries.min(1 << 22));
  for _ in 0..entries {
    ids.push(reader.u32()?);
  }
  Ok((offsets, ids))
}

pub(crate) fn encode_coverage(path_count: u32, bits: &[u8]) -> Vec<u8> {
  let mut writer = Writer::new(TAG_COVERAGE, path_count);
  writer.bytes(bits);
  writer.into_bytes()
}

pub(crate) fn decode_coverage(bytes: &[u8]) -> Result<(u32, Vec<u8>)> {
  let mut reader = Reader::new(bytes, TAG_COVERAGE, "coverage.bin")?;
  let path_count = reader.u32()?;
  Ok((path_count, reader.rest().to_vec()))
}
