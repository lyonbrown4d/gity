//! Human-readable names from SCIP symbol strings.
//!
//! A SCIP symbol is a machine key such as
//! `rust-analyzer cargo app 1.0.0 app/Config#new().`, made of a package header
//! followed by a descriptor chain. Editing surfaces need `new`, `Config` and
//! `app` instead, so this module parses the chain and degrades to the raw string
//! whenever the input does not follow the grammar.

/// Readable parts of one SCIP symbol string.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SymbolName {
  /// The symbol string as received; always usable as a fallback key.
  pub raw: String,
  /// Last descriptor (`new`), or the whole raw string when unparseable.
  pub display_name: String,
  /// Enclosing descriptors (`Config`), empty at the package root.
  pub container: String,
  /// Package name from the header (`app`), empty when absent.
  pub package: String,
}

impl SymbolName {
  /// `container::display_name`, or just the display name at the root.
  pub fn path(&self) -> String {
    if self.container.is_empty() {
      self.display_name.clone()
    } else {
      format!("{}::{}", self.container, self.display_name)
    }
  }
}

/// Parses a SCIP symbol string; never panics and never fails.
pub fn symbol_name(raw: &str) -> SymbolName {
  let fields = fields(raw);
  if fields.len() < 5 {
    return malformed(raw);
  }
  let (scheme, manager, package, version) = (&fields[0], &fields[1], &fields[2], &fields[3]);
  let chain = fields[4..].join(" ");
  if scheme.is_empty() || manager.is_empty() || version.is_empty() || chain.is_empty() {
    return malformed(raw);
  }
  let descriptors = descriptors(&chain);
  let Some((_, display_name)) = descriptors.last() else {
    return malformed(raw);
  };
  let preceding = &descriptors[..descriptors.len() - 1];
  // Types and terms read best as the container; a bare module chain is the
  // fallback when the symbol is not nested in a type.
  let typed: Vec<&str> = preceding
    .iter()
    .filter(|(kind, _)| *kind != '/')
    .map(|(_, name)| name.as_str())
    .collect();
  let container = if typed.is_empty() {
    preceding
      .iter()
      .map(|(_, name)| name.as_str())
      .collect::<Vec<&str>>()
      .join("::")
  } else {
    typed.join("::")
  };
  SymbolName {
    raw: raw.to_string(),
    display_name: display_name.clone(),
    container,
    package: unquote(package),
  }
}

fn malformed(raw: &str) -> SymbolName {
  SymbolName {
    raw: raw.to_string(),
    display_name: raw.to_string(),
    ..SymbolName::default()
  }
}

/// Splits the symbol on spaces that are not inside backticks.
fn fields(raw: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut current = String::new();
  let mut quoted = false;
  for c in raw.chars() {
    match c {
      '`' => {
        quoted = !quoted;
        current.push(c);
      }
      ' ' if !quoted => out.push(std::mem::take(&mut current)),
      _ => current.push(c),
    }
  }
  out.push(current);
  out
}

/// Strips the backtick quoting SCIP uses for names with reserved characters.
fn unquote(field: &str) -> String {
  let trimmed = field.trim_matches('`');
  if trimmed.is_empty() {
    field.to_string()
  } else {
    trimmed.to_string()
  }
}

/// Walks the descriptor chain: `<name><marker>` pairs, where the marker names
/// the descriptor's kind (`/` namespace, `#` type, `.` term, `:` meta, `!`
/// macro). Parameters and type parameters are written `(name)` and `[name]`, and
/// a disambiguator such as `(+1)` sits between the name and its marker.
fn descriptors(chain: &str) -> Vec<(char, String)> {
  let chars: Vec<char> = chain.chars().collect();
  let mut out: Vec<(char, String)> = Vec::new();
  let mut index = 0;
  while index < chars.len() {
    let name = read_name(&chars, &mut index);
    if name.is_empty() {
      // A marker with no name in front of it separates descriptors, or starts a
      // parameter descriptor.
      if let Some((kind, inner)) = read_group(&chars, &mut index) {
        if !inner.is_empty() {
          out.push((kind, inner));
        }
      } else {
        index += 1;
      }
      continue;
    }
    // A disambiguator such as `(+1)` sits between the name and its marker.
    read_group(&chars, &mut index);
    let kind = match chars.get(index) {
      Some(&marker) if is_marker(marker) => {
        index += 1;
        marker
      }
      // A chain that stops after a name means a term.
      _ => '.',
    };
    out.push((kind, name));
  }
  out
}

fn is_marker(c: char) -> bool {
  matches!(c, '/' | '#' | '.' | ':' | '!')
}

/// Reads one descriptor name; stops at a marker or a group opener.
fn read_name(chars: &[char], index: &mut usize) -> String {
  let mut name = String::new();
  let mut quoted = false;
  while let Some(&c) = chars.get(*index) {
    match c {
      '`' => {
        quoted = !quoted;
        *index += 1;
      }
      _ if !quoted && (is_marker(c) || c == '(' || c == '[') => break,
      _ => {
        name.push(c);
        *index += 1;
      }
    }
  }
  name
}

/// Reads `(inner)` or `[inner]`, returning the kind and the inner text.
fn read_group(chars: &[char], index: &mut usize) -> Option<(char, String)> {
  let opener = *chars.get(*index)?;
  let closer = match opener {
    '(' => ')',
    '[' => ']',
    _ => return None,
  };
  *index += 1;
  let mut inner = String::new();
  while let Some(&c) = chars.get(*index) {
    *index += 1;
    if c == closer {
      break;
    }
    if c != '`' {
      inner.push(c);
    }
  }
  Some((opener, inner))
}
