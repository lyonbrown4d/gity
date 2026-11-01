//! mdBook mermaid preprocessor built from the workspace, so contributors and
//! CI do not need a globally installed `mdbook-mermaid`.
use anyhow::{Context, Result, bail};
use mdbook_mermaid::Mermaid;
use mdbook_preprocessor::Preprocessor;
use std::io;

/// Arguments mdBook forwards to a preprocessor: `supports <renderer>` for the
/// capability probe, otherwise the book arrives on stdin.
pub fn run(args: &[String]) -> Result<()> {
  match args.first().map(String::as_str) {
    Some("supports") => supports(args.get(1)),
    _ => preprocess(),
  }
}

fn supports(renderer: Option<&String>) -> Result<()> {
  let renderer = renderer.context("mdBook must pass the renderer to `supports`")?;
  if !Mermaid.supports_renderer(renderer)? {
    bail!("mermaid does not support the {renderer} renderer");
  }
  Ok(())
}

fn preprocess() -> Result<()> {
  let (context, book) = mdbook_preprocessor::parse_input(io::stdin())?;
  if context.mdbook_version != mdbook_preprocessor::MDBOOK_VERSION {
    eprintln!(
      "warning: the mermaid preprocessor was built against mdBook {} but is running under {}",
      mdbook_preprocessor::MDBOOK_VERSION,
      context.mdbook_version
    );
  }
  let book = Mermaid.run(&context, book)?;
  serde_json::to_writer(io::stdout(), &book)?;
  Ok(())
}
