//! Display names from SCIP symbol strings: Rust, TypeScript and junk.

use gity_scip::{SymbolName, symbol_name};

fn named(raw: &str, display: &str, container: &str, package: &str) -> SymbolName {
  SymbolName {
    raw: raw.to_string(),
    display_name: display.to_string(),
    container: container.to_string(),
    package: package.to_string(),
  }
}

#[test]
fn rust_method_symbol_exposes_type_and_package() {
  assert_eq!(
    symbol_name("rust-analyzer cargo app 1.0 app/Config#new()."),
    named(
      "rust-analyzer cargo app 1.0 app/Config#new().",
      "new",
      "Config",
      "app"
    )
  );
}

#[test]
fn rust_type_symbol_uses_its_module_as_container() {
  assert_eq!(
    symbol_name("rust-analyzer cargo app 1.0 app/Config#"),
    named(
      "rust-analyzer cargo app 1.0 app/Config#",
      "Config",
      "app",
      "app"
    )
  );
}

#[test]
fn rust_free_function_is_a_term_descriptor() {
  assert_eq!(
    symbol_name("rust-analyzer cargo app 1.0 app/config/parse().").display_name,
    "parse"
  );
}

#[test]
fn typescript_symbol_unquotes_file_names() {
  let raw = "scip-typescript npm @scope/pkg 1.0.0 src/`file.ts`/Foo#bar().";
  let name = symbol_name(raw);
  assert_eq!(name.display_name, "bar");
  assert_eq!(name.container, "Foo");
  assert_eq!(name.package, "@scope/pkg");
  assert_eq!(name.path(), "Foo::bar");
}

#[test]
fn typescript_quoted_package_survives_spaces() {
  // A file name with a dot must be backtick-quoted; otherwise `.ts` reads as a
  // descriptor marker.
  let raw = "scip-typescript npm `my package` 2.0.0 src/`index.ts`/Widget#render().";
  let name = symbol_name(raw);
  assert_eq!(name.package, "my package");
  assert_eq!(name.display_name, "render");
  assert_eq!(name.container, "Widget");
}

#[test]
fn path_is_the_display_name_at_the_package_root() {
  let name = symbol_name("rust-analyzer cargo app 1.0 app/Config#");
  assert_eq!(name.path(), "app::Config");
}

#[test]
fn parameter_and_type_parameter_descriptors_are_names_too() {
  assert_eq!(
    symbol_name("rust-analyzer cargo app 1.0 app/Foo#bar().(value)").display_name,
    "value"
  );
  assert_eq!(
    symbol_name("rust-analyzer cargo app 1.0 app/Foo#bar().[T]").display_name,
    "T"
  );
}

#[test]
fn disambiguators_do_not_leak_into_names() {
  assert_eq!(
    symbol_name("rust-analyzer cargo app 1.0 app/Foo#bar(+1).").display_name,
    "bar"
  );
}

#[test]
fn malformed_symbols_fall_back_to_the_raw_string() {
  let cases = [
    "",
    "local 42",
    "not a scip symbol",
    "rust-analyzer cargo app 1.0",
    "rust-analyzer cargo app 1.0 ",
  ];
  for raw in cases {
    let name = symbol_name(raw);
    assert_eq!(name.raw, raw);
    assert_eq!(name.display_name, raw, "{raw:?} must not invent a name");
    assert!(name.container.is_empty(), "{raw:?}");
    assert!(name.package.is_empty(), "{raw:?}");
    assert_eq!(name.path(), name.display_name);
  }
}

#[test]
fn empty_descriptor_chain_still_yields_a_usable_name() {
  // Terminators only: no descriptor carries a name, so the raw string is used.
  let raw = "rust-analyzer cargo app 1.0 .";
  let name = symbol_name(raw);
  assert_eq!(name.display_name, raw);
  assert!(name.container.is_empty());
}

#[test]
fn deep_module_chain_uses_the_nearest_type_as_container() {
  let raw = "scip-rust cargo serde 1.0.0 serde/de/Deserialize#deserialize().";
  let name = symbol_name(raw);
  assert_eq!(name.display_name, "deserialize");
  assert_eq!(name.container, "Deserialize");
  assert_eq!(name.package, "serde");
}
