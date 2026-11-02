//! The two-index structures read straight out of their mappings: the CSR that
//! maps a symbol to occurrences, and the coverage bitset.
//!
//! Both are accessed through computed offsets, so a lookup touches one page
//! instead of loading the table.

use crate::mapped::Mapped;
use crate::tables::{TAG_COVERAGE, TAG_REFS};
use crate::{Error, Result};
use std::path::Path;

/// `refs.bin`: symbol count, entry count, `symbols + 1` offsets, then entries.
#[derive(Debug)]
pub(crate) struct Refs {
  mapped: Mapped,
  symbols: u32,
  entries: u32,
  offsets_at: usize,
  entries_at: usize,
}

impl Refs {
  pub(crate) fn open(path: &Path) -> Result<Refs> {
    let table = "refs.bin";
    let mapped = Mapped::open(path, table)?;
    if !mapped.has_tag(TAG_REFS) {
      return Err(Error::Corrupt { table });
    }
    let symbols = mapped.u32_at(4).ok_or(Error::Corrupt { table })?;
    let entries = mapped.u32_at(8).ok_or(Error::Corrupt { table })?;
    let offsets_at = 12;
    let entries_at = (symbols as usize)
      .checked_add(1)
      .and_then(|count| count.checked_mul(4))
      .and_then(|size| size.checked_add(offsets_at))
      .ok_or(Error::Corrupt { table })?;
    let end = (entries as usize)
      .checked_mul(4)
      .and_then(|size| size.checked_add(entries_at))
      .ok_or(Error::Corrupt { table })?;
    if end > mapped.len() {
      return Err(Error::Corrupt { table });
    }
    let refs = Refs {
      mapped,
      symbols,
      entries,
      offsets_at,
      entries_at,
    };
    if !refs.offsets_are_sorted() {
      return Err(Error::Corrupt { table });
    }
    Ok(refs)
  }

  pub(crate) fn symbols(&self) -> u32 {
    self.symbols
  }

  pub(crate) fn entries(&self) -> u32 {
    self.entries
  }

  /// One CSR offset; index runs up to and including `symbols`.
  pub(crate) fn offset(&self, index: u32) -> Option<u32> {
    if index > self.symbols {
      return None;
    }
    self.mapped.u32_at(
      self
        .offsets_at
        .checked_add((index as usize).checked_mul(4)?)?,
    )
  }

  pub(crate) fn last_offset(&self) -> Option<u32> {
    self.offset(self.symbols)
  }

  /// Index range of one symbol's CSR row, `None` past the last symbol.
  pub(crate) fn row(&self, symbol: u32) -> Option<std::ops::Range<u32>> {
    let start = self.offset(symbol)?;
    let end = self.offset(symbol.checked_add(1)?)?;
    (start <= end && end <= self.entries).then_some(start..end)
  }

  pub(crate) fn entry(&self, index: u32) -> Option<u32> {
    if index >= self.entries {
      return None;
    }
    self.mapped.u32_at(
      self
        .entries_at
        .checked_add((index as usize).checked_mul(4)?)?,
    )
  }

  fn offsets_are_sorted(&self) -> bool {
    let mut previous = 0;
    for index in 0..=self.symbols {
      let Some(offset) = self.offset(index) else {
        return false;
      };
      if offset < previous || offset > self.entries {
        return false;
      }
      previous = offset;
    }
    true
  }

  pub(crate) fn mapped_bytes(&self) -> usize {
    self.mapped.mapped_bytes()
  }

  pub(crate) fn shrink(&self) -> Option<(u64, u64)> {
    self.mapped.shrink()
  }
}

/// `coverage.bin`: path count then one bit per path, MSB-first.
#[derive(Debug)]
pub(crate) struct Bits {
  mapped: Mapped,
  paths: u32,
}

impl Bits {
  pub(crate) fn open(path: &Path) -> Result<Bits> {
    let table = "coverage.bin";
    let mapped = Mapped::open(path, table)?;
    if !mapped.has_tag(TAG_COVERAGE) {
      return Err(Error::Corrupt { table });
    }
    let paths = mapped.u32_at(4).ok_or(Error::Corrupt { table })?;
    let length = (paths as usize).div_ceil(8);
    if length.checked_add(8).is_none_or(|end| end > mapped.len()) {
      return Err(Error::Corrupt { table });
    }
    Ok(Bits { mapped, paths })
  }

  pub(crate) fn paths(&self) -> u32 {
    self.paths
  }

  pub(crate) fn contains(&self, index: u32) -> bool {
    if index >= self.paths {
      return false;
    }
    let byte = index as usize / 8;
    let Some(bits) = self.mapped.bytes().get(8 + byte) else {
      return false;
    };
    bits & (0x80 >> (index % 8)) != 0
  }

  pub(crate) fn mapped_bytes(&self) -> usize {
    self.mapped.mapped_bytes()
  }

  pub(crate) fn shrink(&self) -> Option<(u64, u64)> {
    self.mapped.shrink()
  }
}
