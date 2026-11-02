//! Output helpers: tables for humans, raw JSON for scripts.
use serde_json::Value;
use std::io::IsTerminal;

pub fn print_json(value: &Value) {
  match serde_json::to_string_pretty(value) {
    Ok(text) => println!("{text}"),
    Err(_) => println!("{value}"),
  }
}

/// Renders rows with aligned columns; widths follow the widest cell so the
/// output stays readable without pulling in a table crate.
pub fn table(headers: &[&str], rows: &[Vec<String>]) {
  if rows.is_empty() {
    println!("(no results)");
    return;
  }
  let mut widths: Vec<usize> = headers
    .iter()
    .map(|header| header.chars().count())
    .collect();
  for row in rows {
    for (index, cell) in row.iter().enumerate() {
      if index < widths.len() {
        widths[index] = widths[index].max(cell.chars().count());
      }
    }
  }
  println!("{}", line(headers.iter().copied(), &widths));
  for row in rows {
    println!("{}", line(row.iter().map(String::as_str), &widths));
  }
}

fn line<'a>(cells: impl Iterator<Item = &'a str>, widths: &[usize]) -> String {
  let mut out = String::new();
  for (index, cell) in cells.enumerate() {
    if index > 0 {
      out.push_str("  ");
    }
    out.push_str(cell);
    if let Some(width) = widths.get(index) {
      let pad = width.saturating_sub(cell.chars().count());
      out.push_str(&" ".repeat(pad));
    }
  }
  out.trim_end().to_owned()
}

/// Truncates long free-form cells such as titles so rows stay on one line.
pub fn ellipsis(value: &Value, limit: usize) -> String {
  let text = match value {
    Value::String(text) => text.clone(),
    Value::Null => String::new(),
    other => other.to_string(),
  };
  let text = text.replace('\n', " ");
  if text.chars().count() <= limit {
    return text;
  }
  text
    .chars()
    .take(limit.saturating_sub(1))
    .collect::<String>()
    + "…"
}

pub fn field<'a>(row: &'a Value, key: &str) -> &'a Value {
  row.get(key).unwrap_or(&Value::Null)
}

pub fn text(row: &Value, key: &str) -> String {
  match field(row, key) {
    Value::String(text) => text.clone(),
    Value::Null => String::new(),
    other => other.to_string(),
  }
}

/// Colour is only applied when stdout is a terminal.
pub fn color() -> bool {
  std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

pub fn paint(value: &str, code: &str) -> String {
  if color() {
    format!("\u{1b}[{code}m{value}\u{1b}[0m")
  } else {
    value.to_owned()
  }
}
