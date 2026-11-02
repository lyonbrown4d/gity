//! Neutral archive format exchanged between the indexer job and the server.
//!
//! The payload is msgpack compressed with zstd: compact, self-describing, and
//! free of any dependency on the index engine version, so the server can rebuild
//! its local index without re-running the job.
use crate::symbol::SymbolFile;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

/// Bumped whenever the payload shape changes incompatibly.
pub const FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Archive {
  pub version: u32,
  pub commit: String,
  #[serde(default)]
  pub files: Vec<SymbolFile>,
}

impl Archive {
  pub fn new(commit: impl Into<String>, files: Vec<SymbolFile>) -> Self {
    Self {
      version: FORMAT_VERSION,
      commit: commit.into(),
      files,
    }
  }
  pub fn definition_count(&self) -> usize {
    self.files.iter().map(|file| file.definitions.len()).sum()
  }
}

pub fn encode(archive: &Archive) -> Result<Vec<u8>> {
  let payload = rmp_serde::to_vec_named(archive).context("encode symbol archive")?;
  zstd::encode_all(payload.as_slice(), 9).context("compress symbol archive")
}

pub fn decode(bytes: &[u8]) -> Result<Archive> {
  let payload = zstd::decode_all(bytes).context("decompress symbol archive")?;
  let archive: Archive = rmp_serde::from_slice(&payload).context("decode symbol archive")?;
  if archive.version != FORMAT_VERSION {
    bail!(
      "symbol archive version {} is not supported (expected {FORMAT_VERSION})",
      archive.version
    );
  }
  Ok(archive)
}
