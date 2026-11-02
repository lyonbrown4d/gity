//! Fixed-layout binary primitives shared by every shard table.
//!
//! Tables are little-endian, start with a four-byte tag and a `u32` record
//! count. Writing goes through [`Writer`]; reading slices mapped bytes directly
//! (see [`crate::mapped`]), so there is no cursor to keep in sync.

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

/// Slices a pool at a recorded offset, or `None` when the record points outside.
pub(crate) fn pool_str(pool: &[u8], offset: u32, length: u32) -> Option<&str> {
  let start = usize::try_from(offset).ok()?;
  let end = start.checked_add(usize::try_from(length).ok()?)?;
  let chunk = pool.get(start..end)?;
  std::str::from_utf8(chunk).ok()
}
