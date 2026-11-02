//! Shared compile-time translation assets for browser and HTTP request contexts.
//!
//! Every lookup takes its locale explicitly. This crate never changes the
//! process-wide rust-i18n locale, so concurrent users cannot affect one another.
rust_i18n::i18n!("locales", fallback = "en");

/// Normalize browser language tags without retaining request-global state.
pub fn normalize_locale(locale: &str) -> &'static str {
  if locale
    .split(['-', '_'])
    .next()
    .is_some_and(|tag| tag.eq_ignore_ascii_case("zh"))
  {
    "zh"
  } else {
    "en"
  }
}

/// Translate an original English phrase or catalog key with named values.
///
/// Original `{name}` templates remain compatible. Locale assets use the
/// library's `%{name}` interpolation syntax internally.
pub fn translate(key: &str, locale: &str, args: &[(&str, &str)]) -> String {
  let localized = rust_i18n::t!(key, locale = normalize_locale(locale)).to_string();
  if localized == key && args.is_empty() {
    return localized;
  }
  let mut names = args.iter().map(|(name, _)| *name).collect::<Vec<_>>();
  let mut values = args
    .iter()
    .map(|(_, value)| value.to_string())
    .collect::<Vec<_>>();
  // Fill unresolved catalog markers with the legacy spelling in the same
  // interpolation pass. Argument text itself must never be rewritten.
  for fragment in localized.split("%{").skip(1) {
    if let Some((name, _)) = fragment.split_once('}')
      && !names.contains(&name)
    {
      names.push(name);
      values.push(format!("{{{name}}}"));
    }
  }
  rust_i18n::replace_patterns(&localized, &names, &values)
}

/// Localize a known server message and preserve unknown dynamic messages.
pub fn translate_message(locale: &str, message: &str) -> String {
  translate(message, locale, &[])
}

// Track every locale asset in native and WASM compiler dependency graphs.
mod assets;
const _: &[&str] = assets::CHUNKS;
#[cfg(test)]
#[path = "tests/server_catalog.rs"]
mod server_catalog;
#[cfg(test)]
#[path = "tests/catalog.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/frontend_catalog.rs"]
mod frontend_catalog;
