//! The on-disk shard: six fixed-layout tables plus the manifest describing them.
//!
//! Layout inside one shard directory, next to `gity-code-intel`'s lexical
//! snapshot so an operator only learns one convention:
//!
//! ```text
//! <dir>/manifest.json   commit, indexer, format version, counts, coverage
//! <dir>/paths.bin       sorted paths, each with its occurrence range
//! <dir>/occ.bin         fixed records, ordered by path then position
//! <dir>/symbols.bin     sorted symbol strings, kind, doc/signature offsets
//! <dir>/docs.bin        documentation and signature text, flat
//! <dir>/refs.bin        CSR: symbol -> occurrences
//! <dir>/coverage.bin    bitset of precisely analysed paths
//! <dir>/lock            writer/reader coordination
//! ```
//!
//! The manifest is written last and is the commit point. Reads never fail on a
//! missing or damaged shard: [`Shard::open`] answers `Ok(None)` so the caller
//! degrades to the lexical layer instead of returning an error to a user.

use crate::assemble::{Raw, Tables};
use crate::model::Decoded;
use crate::tables;
use crate::{Error, FORMAT_VERSION, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};

const MANIFEST: &str = "manifest.json";
const LOCK: &str = "lock";
const PATHS: &str = "paths.bin";
const OCC: &str = "occ.bin";
const SYMBOLS: &str = "symbols.bin";
const DOCS: &str = "docs.bin";
const REFS: &str = "refs.bin";
const COVERAGE: &str = "coverage.bin";

/// What a shard holds; the analogue of the lexical layer's `status.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
  pub format_version: u32,
  pub commit: String,
  pub indexer: String,
  pub indexer_version: String,
  /// RFC 3339, same spelling as the lexical status document.
  pub indexed_at: String,
  pub counts: Counts,
  pub coverage: CoverageSummary,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counts {
  pub paths: usize,
  pub occurrences: usize,
  pub symbols: usize,
  /// CSR entries: occurrences that carry a symbol.
  pub references: usize,
  pub doc_bytes: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageSummary {
  pub total_paths: usize,
  pub precise_paths: usize,
  /// Paths the runner reported but the indexer did not analyse.
  pub listed_paths: usize,
}

/// An open shard: tables resident in memory, manifest for the caller.
pub struct Shard {
  dir: PathBuf,
  manifest: Manifest,
  tables: Tables,
}

impl Shard {
  /// Rebuilds the shard from a decoded index, replacing any previous snapshot.
  ///
  /// Only one writer may build a directory at a time; the lock is advisory and
  /// released when this call returns.
  pub fn build(dir: impl AsRef<Path>, decoded: &Decoded) -> Result<Manifest> {
    let dir = dir.as_ref();
    let tables = Tables::from_decoded(decoded)?;
    std::fs::create_dir_all(dir).map_err(|source| Error::Io {
      path: dir.display().to_string(),
      source,
    })?;
    let _lock = Lock::acquire(dir, true)?;
    let encoded = [
      (
        PATHS,
        tables::encode_paths(&tables.paths, &tables.path_pool),
      ),
      (OCC, tables::encode_occ(&tables.occ)),
      (
        SYMBOLS,
        tables::encode_symbols(&tables.symbols, &tables.symbol_pool),
      ),
      (DOCS, tables::encode_docs(&tables.docs)),
      (
        REFS,
        tables::encode_refs(&tables.ref_offsets, &tables.ref_entries),
      ),
      (
        COVERAGE,
        tables::encode_coverage(tables.paths.len() as u32, &tables.coverage),
      ),
    ];
    for (name, bytes) in encoded {
      write_atomic(dir, name, &bytes)?;
    }
    let counts = tables.counts();
    let (precise, listed) = tables.coverage_summary();
    let manifest = Manifest {
      format_version: FORMAT_VERSION,
      commit: decoded.commit.clone(),
      indexer: decoded.indexer.clone(),
      indexer_version: decoded.indexer_version.clone(),
      indexed_at: timestamp(),
      counts,
      coverage: CoverageSummary {
        total_paths: tables.counts().paths,
        precise_paths: precise,
        listed_paths: listed,
      },
    };
    let text = serde_json::to_vec_pretty(&manifest)?;
    write_atomic(dir, MANIFEST, &text)?;
    Ok(manifest)
  }

  /// Opens a shard, or `Ok(None)` when there is nothing usable to read.
  pub fn open(dir: impl AsRef<Path>) -> Result<Option<Shard>> {
    let dir = dir.as_ref().to_path_buf();
    let Some(manifest) = read_manifest(&dir) else {
      return Ok(None);
    };
    if manifest.format_version != FORMAT_VERSION {
      return Ok(None);
    }
    let Ok(_lock) = Lock::acquire(&dir, false) else {
      return Ok(None);
    };
    let Some(raw) = read_tables(&dir) else {
      return Ok(None);
    };
    let Ok(tables) = Tables::decode(raw) else {
      return Ok(None);
    };
    Ok(Some(Shard {
      dir,
      manifest,
      tables,
    }))
  }

  pub fn manifest(&self) -> &Manifest {
    &self.manifest
  }

  pub fn dir(&self) -> &Path {
    &self.dir
  }

  pub(crate) fn tables(&self) -> &Tables {
    &self.tables
  }
}

fn write_atomic(dir: &Path, name: &str, bytes: &[u8]) -> Result<()> {
  let temporary = dir.join(format!("{name}.new"));
  let io = |source| Error::Io {
    path: temporary.display().to_string(),
    source,
  };
  std::fs::write(&temporary, bytes).map_err(io)?;
  std::fs::rename(&temporary, dir.join(name)).map_err(|source| Error::Io {
    path: dir.join(name).display().to_string(),
    source,
  })
}

fn read_manifest(dir: &Path) -> Option<Manifest> {
  let text = std::fs::read_to_string(dir.join(MANIFEST)).ok()?;
  serde_json::from_str(&text).ok()
}

/// Reads all six tables while the caller holds the shard lock.
fn read_tables(dir: &Path) -> Option<Raw> {
  let raw = Raw {
    paths: std::fs::read(dir.join(PATHS)).ok()?,
    occ: std::fs::read(dir.join(OCC)).ok()?,
    symbols: std::fs::read(dir.join(SYMBOLS)).ok()?,
    docs: std::fs::read(dir.join(DOCS)).ok()?,
    refs: std::fs::read(dir.join(REFS)).ok()?,
    coverage: std::fs::read(dir.join(COVERAGE)).ok()?,
  };
  Some(raw)
}

fn timestamp() -> String {
  chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// Writer/reader lock, released when the file handle drops.
struct Lock {
  #[allow(dead_code)]
  file: File,
}

impl Lock {
  fn acquire(dir: &Path, exclusive: bool) -> Result<Lock> {
    let path = dir.join(LOCK);
    let file = std::fs::OpenOptions::new()
      .create(true)
      .truncate(false)
      .read(true)
      .write(true)
      .open(&path)
      .map_err(|source| Error::Io {
        path: path.display().to_string(),
        source,
      })?;
    let locked = if exclusive {
      fs2::FileExt::try_lock_exclusive(&file)
    } else {
      fs2::FileExt::try_lock_shared(&file)
    };
    locked.map_err(|_| Error::Locked(path.display().to_string()))?;
    Ok(Lock { file })
  }
}
