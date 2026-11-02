//! Symbol-side accessors: the repository symbol table, the external table, and
//! the CSR that maps a symbol to the occurrences naming it.
//!
//! Both tables are sorted by symbol string, so a lookup is a binary search over
//! mapped records. They are separate tables on purpose: only repository symbols
//! own an ID that occurrences and the CSR reference, so an external symbol can
//! never create a phantom reference or a phantom definition.

use crate::format::pool_str;
use crate::mapped::Table;
use crate::model::SymbolKind;
use crate::read::Store;
use crate::tables::{SymbolRecord, parse_symbol};

/// View of one symbol table plus its string pool.
struct Symbols<'a> {
  table: &'a Table,
  pool: &'a [u8],
}

impl<'a> Symbols<'a> {
  fn new(table: &'a Table) -> Self {
    Self {
      table,
      pool: table.pool(),
    }
  }

  fn record(&self, id: u32) -> Option<SymbolRecord> {
    parse_symbol(self.table.record(id)?)
  }

  fn text(&self, id: u32) -> Option<&'a str> {
    let record = self.record(id)?;
    pool_str(self.pool, record.text_off, record.text_len)
  }

  fn find(&self, symbol: &str) -> Option<u32> {
    let mut low = 0u32;
    let mut high = self.table.count();
    while low < high {
      let middle = low + (high - low) / 2;
      match self.text(middle)?.cmp(symbol) {
        std::cmp::Ordering::Less => low = middle + 1,
        std::cmp::Ordering::Greater => high = middle,
        std::cmp::Ordering::Equal => return Some(middle),
      }
    }
    None
  }
}

impl Store {
  pub(crate) fn symbol(&self, symbol_id: u32) -> Option<&str> {
    Symbols::new(&self.symbols).text(symbol_id)
  }

  pub(crate) fn find_symbol(&self, symbol: &str) -> Option<u32> {
    Symbols::new(&self.symbols).find(symbol)
  }

  pub(crate) fn kind(&self, symbol_id: u32) -> SymbolKind {
    kind_of(Symbols::new(&self.symbols).record(symbol_id))
  }

  pub(crate) fn documentation(&self, symbol_id: u32) -> &str {
    documentation_of(
      self.docs.bytes(),
      Symbols::new(&self.symbols).record(symbol_id),
    )
  }

  pub(crate) fn signature(&self, symbol_id: u32) -> &str {
    signature_of(
      self.docs.bytes(),
      Symbols::new(&self.symbols).record(symbol_id),
    )
  }

  pub(crate) fn find_external(&self, symbol: &str) -> Option<u32> {
    Symbols::new(&self.external).find(symbol)
  }

  pub(crate) fn external_kind(&self, external_id: u32) -> SymbolKind {
    kind_of(Symbols::new(&self.external).record(external_id))
  }

  pub(crate) fn external_documentation(&self, external_id: u32) -> &str {
    documentation_of(
      self.docs.bytes(),
      Symbols::new(&self.external).record(external_id),
    )
  }

  pub(crate) fn external_signature(&self, external_id: u32) -> &str {
    signature_of(
      self.docs.bytes(),
      Symbols::new(&self.external).record(external_id),
    )
  }

  /// Global occurrence IDs naming one symbol, in repository order.
  pub(crate) fn occurrences_of_symbol(&self, symbol_id: u32) -> impl Iterator<Item = u32> + '_ {
    let range = self.refs.row(symbol_id).unwrap_or(0..0);
    range.filter_map(move |index| self.refs.entry(index))
  }
}

fn kind_of(record: Option<SymbolRecord>) -> SymbolKind {
  record.map_or(SymbolKind::default(), |record| {
    SymbolKind::from_code(record.kind as i32)
  })
}

fn documentation_of(docs: &[u8], record: Option<SymbolRecord>) -> &str {
  record
    .and_then(|record| pool_str(docs, record.doc_off, record.doc_len))
    .unwrap_or("")
}

fn signature_of(docs: &[u8], record: Option<SymbolRecord>) -> &str {
  record
    .and_then(|record| pool_str(docs, record.signature_off, record.signature_len))
    .unwrap_or("")
}
