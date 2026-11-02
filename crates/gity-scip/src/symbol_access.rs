//! Symbol-side accessors: the repository symbol table, the external table, and
//! the CSR that maps a symbol to the occurrences naming it.
//!
//! Both tables are sorted by symbol string, so a lookup is a binary search. They
//! are separate tables on purpose: only repository symbols own an ID that
//! occurrences and the CSR reference, so an external symbol can never create a
//! phantom reference or a phantom definition.

use crate::assemble::Tables;
use crate::format::pool_str;
use crate::model::SymbolKind;
use crate::tables::SymbolRecord;

impl Tables {
  pub(crate) fn symbol(&self, symbol_id: u32) -> Option<&str> {
    text_of(&self.symbol_pool, self.symbols.get(symbol_id as usize)?)
  }

  pub(crate) fn find_symbol(&self, symbol: &str) -> Option<u32> {
    find_in(&self.symbols, &self.symbol_pool, symbol)
  }

  pub(crate) fn kind(&self, symbol_id: u32) -> SymbolKind {
    kind_of(self.symbols.get(symbol_id as usize))
  }

  pub(crate) fn documentation(&self, symbol_id: u32) -> &str {
    documentation_of(&self.docs, self.symbols.get(symbol_id as usize))
  }

  pub(crate) fn signature(&self, symbol_id: u32) -> &str {
    signature_of(&self.docs, self.symbols.get(symbol_id as usize))
  }

  pub(crate) fn find_external(&self, symbol: &str) -> Option<u32> {
    find_in(&self.external, &self.external_pool, symbol)
  }

  pub(crate) fn external_kind(&self, external_id: u32) -> SymbolKind {
    kind_of(self.external.get(external_id as usize))
  }

  pub(crate) fn external_documentation(&self, external_id: u32) -> &str {
    documentation_of(&self.docs, self.external.get(external_id as usize))
  }

  pub(crate) fn external_signature(&self, external_id: u32) -> &str {
    signature_of(&self.docs, self.external.get(external_id as usize))
  }

  /// CSR slice for one symbol: global occurrence IDs in repository order.
  pub(crate) fn occurrences_of_symbol(&self, symbol_id: u32) -> &[u32] {
    let index = symbol_id as usize;
    let Some(start) = self.ref_offsets.get(index).copied() else {
      return &[];
    };
    let Some(end) = self.ref_offsets.get(index + 1).copied() else {
      return &[];
    };
    self
      .ref_entries
      .get(start as usize..end as usize)
      .unwrap_or(&[])
  }
}

fn text_of<'a>(pool: &'a [u8], record: &SymbolRecord) -> Option<&'a str> {
  pool_str(pool, record.text_off, record.text_len)
}

fn find_in(records: &[SymbolRecord], pool: &[u8], symbol: &str) -> Option<u32> {
  let found = records.binary_search_by(|record| {
    pool_str(pool, record.text_off, record.text_len)
      .unwrap_or("")
      .cmp(symbol)
  });
  found.ok().map(|index| index as u32)
}

fn kind_of(record: Option<&SymbolRecord>) -> SymbolKind {
  record.map_or(SymbolKind::default(), |record| {
    SymbolKind::from_code(record.kind as i32)
  })
}

fn documentation_of<'a>(docs: &'a [u8], record: Option<&SymbolRecord>) -> &'a str {
  record
    .and_then(|record| pool_str(docs, record.doc_off, record.doc_len))
    .unwrap_or("")
}

fn signature_of<'a>(docs: &'a [u8], record: Option<&SymbolRecord>) -> &'a str {
  record
    .and_then(|record| pool_str(docs, record.signature_off, record.signature_len))
    .unwrap_or("")
}
