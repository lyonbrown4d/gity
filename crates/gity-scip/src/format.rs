//! Fixed-layout binary primitives shared by every shard table.
//!
//! Tables are little-endian, start with a four-byte tag and a `u32` record
//! count, and are read back with bounds checks so a truncated or foreign file
//! becomes an error instead of a panic.

use crate::{Error, Result};

/// Builds one table: tag, count, fixed records, then a variable-length pool.
pub(crate) struct Writer {
  bytes: Vec<u8>,
}

impl Writer {
  pub(crate) fn new(tag: [u8; 4], count: u32) -> Self {
    let mut bytes = Vec::with_capacity(64);
    bytes.extend_from_slice(&tag);
    bytes.extend_from_slice(&count.to_le_bytes());
    Self { bytes }
  }

  pub(crate) fn u32(&mut self, value: u32) {
    self.bytes.extend_from_slice(&value.to_le_bytes());
  }

  pub(crate) fn bytes(&mut self, bytes: &[u8]) {
    self.bytes.extend_from_slice(bytes);
  }

  pub(crate) fn into_bytes(self) -> Vec<u8> {
    self.bytes
  }
}

/// Reads one table written by [`Writer`].
pub(crate) struct Reader<'a> {
  bytes: &'a [u8],
  position: usize,
  table: &'static str,
}

impl<'a> Reader<'a> {
  pub(crate) fn new(bytes: &'a [u8], tag: [u8; 4], table: &'static str) -> Result<Self> {
    if bytes.len() < 8 || bytes[..4] != tag {
      return Err(Error::Corrupt { table });
    }
    Ok(Self {
      bytes,
      position: 4,
      table,
    })
  }

  pub(crate) fn u32(&mut self) -> Result<u32> {
    let chunk = self.take(4)?;
    let mut word = [0u8; 4];
    word.copy_from_slice(chunk);
    Ok(u32::from_le_bytes(word))
  }

  pub(crate) fn slice(&mut self, length: usize) -> Result<&'a [u8]> {
    self.take(length)
  }

  /// Everything left in the table, used for string and document pools.
  pub(crate) fn rest(&mut self) -> &'a [u8] {
    let rest = &self.bytes[self.position.min(self.bytes.len())..];
    self.position = self.bytes.len();
    rest
  }

  fn take(&mut self, length: usize) -> Result<&'a [u8]> {
    let end = self
      .position
      .checked_add(length)
      .ok_or(Error::Corrupt { table: self.table })?;
    if end > self.bytes.len() {
      return Err(Error::Corrupt { table: self.table });
    }
    let chunk = &self.bytes[self.position..end];
    self.position = end;
    Ok(chunk)
  }
}

/// Slices a pool at a recorded offset, or `None` when the record points outside.
pub(crate) fn pool_str(pool: &[u8], offset: u32, length: u32) -> Option<&str> {
  let start = usize::try_from(offset).ok()?;
  let end = start.checked_add(usize::try_from(length).ok()?)?;
  let chunk = pool.get(start..end)?;
  std::str::from_utf8(chunk).ok()
}
