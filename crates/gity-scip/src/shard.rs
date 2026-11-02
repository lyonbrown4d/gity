//! The on-disk shard: eight fixed-layout tables plus the manifest describing
//! them, opened by mapping the files rather than reading them.
//!
//! Layout inside one shard directory, next to `gity-code-intel`'s lexical
//! snapshot so an operator only learns one convention:
//!
//! ```text
//! <dir>/manifest.json   commit, indexer, format version, encoding, counts
//! <dir>/paths.bin       sorted paths, each with its occurrence range
//! <dir>/occ.bin         fixed records, ordered by path then position
//! <dir>/symbols.bin     sorted repository symbols: kind, doc/signature offsets
//! <dir>/external.bin    same records for symbols defined in dependencies
//! <dir>/texts.bin       source text per path, when the indexer shipped it
//! <dir>/docs.bin        documentation and signature text, flat
//! <dir>/refs.bin        CSR: symbol -> occurrences
//! <dir>/coverage.bin    bitset of precisely analysed paths
//! <dir>/lock            writer/reader coordination
//! ```
//!
//! The manifest is written last and is the commit point. [`Shard::open`] treats
//! the precise layer as optional and answers `Ok(None)` for anything unusable;
//! [`Shard::open_strict`] reports the concrete reason instead, for operators and
//! tests.

use crate::assemble::Tables;
use crate::manifest::{CoverageSummary, Manifest};
use crate::model::{Decoded, PositionEncoding};
use crate::read::{
  COVERAGE_FILE, DOCS_FILE, EXTERNAL_FILE, OCC_FILE, PATHS_FILE, REFS_FILE, SYMBOLS_FILE, Store,
  TEXTS_FILE,
};
use crate::tables;
use crate::{Error, FORMAT_VERSION, Result};
use std::fs::File;
use std::path::{Path, PathBuf};

const MANIFEST: &str = "manifest.json";
const LOCK: &str = "lock";

/// An open shard: mapped tables and the manifest.
#[derive(Debug)]
pub struct Shard {
  dir: PathBuf,
  manifest: Manifest,
  store: Store,
}

impl Shard {
  /// Rebuilds the shard from a decoded index, replacing any previous snapshot.
  ///
  /// Building is the one path that holds whole tables in memory: it assembles the
  /// records, writes each table to `<name>.new`, renames it into place, and only
  /// then writes the manifest. Readers keep their mappings of the previous files
  /// until they reopen, so a rebuild never disturbs a live request.
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
        PATHS_FILE,
        tables::encode_paths(&tables.paths, &tables.path_pool),
      ),
      (OCC_FILE, tables::encode_occ(&tables.occ)),
      (
        SYMBOLS_FILE,
        tables::encode_symbols(&tables.symbols, &tables.symbol_pool),
      ),
      (
        EXTERNAL_FILE,
        tables::encode_external(&tables.external, &tables.external_pool),
      ),
      (
        TEXTS_FILE,
        tables::encode_texts(&tables.texts, &tables.text_pool),
      ),
      (DOCS_FILE, tables::encode_docs(&tables.docs)),
      (
        REFS_FILE,
        tables::encode_refs(&tables.ref_offsets, &tables.ref_entries),
      ),
      (
        COVERAGE_FILE,
        tables::encode_coverage(tables.paths.len() as u32, &tables.coverage),
      ),
    ];
    for (name, bytes) in encoded {
      write_atomic(dir, name, &bytes)?;
    }
    let (precise, listed) = tables.coverage_summary();
    let manifest = Manifest {
      format_version: FORMAT_VERSION,
      commit: decoded.commit.clone(),
      indexer: decoded.indexer.clone(),
      indexer_version: decoded.indexer_version.clone(),
      indexed_at: timestamp(),
      position_encoding: encoding_summary(&tables),
      counts: tables.counts(),
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

  /// Opens a shard, or `Ok(None)` whenever the precise layer is unusable.
  ///
  /// This is the call site for serving traffic: a missing, truncated, foreign or
  /// busy shard degrades to the lexical layer instead of failing a request.
  pub fn open(dir: impl AsRef<Path>) -> Result<Option<Shard>> {
    Ok(Self::open_strict(dir).unwrap_or(None))
  }

  /// Opens a shard, reporting why an unusable one is unusable.
  ///
  /// Returns `Ok(None)` only when no shard was ever written. Everything else is
  /// an error naming the failure: bad manifest, unsupported format version,
  /// locked directory, missing table, corrupt table, or tables that disagree.
  ///
  /// Opening maps the eight table files and validates their structure; it does
  /// not copy table contents into the heap. The cost is `O(paths + symbols)`
  /// record parses, not `O(occurrences)`, so a caller that serves many requests
  /// should keep the handle and reopen only when the manifest changes.
  pub fn open_strict(dir: impl AsRef<Path>) -> Result<Option<Shard>> {
    let dir = dir.as_ref().to_path_buf();
    let path = dir.join(MANIFEST);
    if !path.exists() {
      return Ok(None);
    }
    let text = std::fs::read_to_string(&path).map_err(|source| Error::Io {
      path: path.display().to_string(),
      source,
    })?;
    let manifest: Manifest = serde_json::from_str(&text)?;
    if manifest.format_version != FORMAT_VERSION {
      return Err(Error::Version {
        found: manifest.format_version,
        expected: FORMAT_VERSION,
      });
    }
    // The lock only covers opening: mappings survive a rebuild, so holding it
    // for the shard's lifetime would block writers for no safety gain.
    let _lock = Lock::acquire(&dir, false)?;
    let store = Store::open(&dir)?;
    Ok(Some(Shard {
      dir,
      manifest,
      store,
    }))
  }

  /// Fails when a mapped file shrank after the shard was opened.
  ///
  /// Reading past the end of a shrunk mapping would fault the process, so every
  /// query asks this first and degrades instead of touching the tables.
  pub fn health(&self) -> Result<()> {
    match self.store.shrink() {
      Some((table, expected, found)) => Err(Error::Stale {
        table,
        expected,
        found,
      }),
      None => Ok(()),
    }
  }

  /// Approximate heap this handle holds.
  ///
  /// Table contents are mapped, not copied, so this is a handful of structs plus
  /// the manifest strings regardless of shard size. Compare with
  /// [`Shard::mapped_bytes`], which reports the file bytes reachable through the
  /// mappings.
  pub fn resident_bytes(&self) -> usize {
    let manifest = &self.manifest;
    std::mem::size_of::<Shard>()
      + self.dir.as_os_str().len()
      + manifest.commit.len()
      + manifest.indexer.len()
      + manifest.indexer_version.len()
      + manifest.indexed_at.len()
      + manifest.position_encoding.len()
  }

  /// Bytes of shard files mapped into this process.
  pub fn mapped_bytes(&self) -> usize {
    self.store.mapped_bytes()
  }

  pub fn manifest(&self) -> &Manifest {
    &self.manifest
  }

  pub fn dir(&self) -> &Path {
    &self.dir
  }

  /// The tables, or `None` when the mapping went stale and must not be read.
  pub(crate) fn store(&self) -> Option<&Store> {
    self.health().ok()?;
    Some(&self.store)
  }
}

/// Dominant column unit of the analysed documents, `mixed` when they disagree.
fn encoding_summary(tables: &Tables) -> String {
  let mut seen: Option<PositionEncoding> = None;
  for path_id in 0..tables.counts().paths as u32 {
    if !tables.is_precise(path_id) {
      continue;
    }
    let encoding = tables.encoding(path_id).unwrap_or_default();
    match seen {
      None => seen = Some(encoding),
      Some(previous) if previous != encoding => return "mixed".to_string(),
      Some(_) => {}
    }
  }
  seen.unwrap_or_default().as_str().to_string()
}

fn write_atomic(dir: &Path, name: &str, bytes: &[u8]) -> Result<()> {
  let temporary = dir.join(format!("{name}.new"));
  let io = |source| Error::Io {
    path: temporary.display().to_string(),
    source,
  };
  std::fs::write(&temporary, bytes).map_err(io)?;
  let target = dir.join(name);
  std::fs::rename(&temporary, &target).map_err(|source| Error::Io {
    path: target.display().to_string(),
    source,
  })
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
