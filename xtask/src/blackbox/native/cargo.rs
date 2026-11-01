use super::super::{Reporter, process};
use super::{Clients, container};
use anyhow::Result;
use std::fs;

pub(super) fn run(context: &Clients, reporter: &mut Reporter) -> Result<()> {
  let root = context.work.join("cargo");
  let publisher = root.join("publisher");
  let consumer = root.join("consumer");
  let cargo_home = root.join("cargo-home");
  fs::create_dir_all(publisher.join("src"))?;
  fs::create_dir_all(consumer.join("src"))?;
  fs::create_dir_all(&cargo_home)?;
  let package = format!("gity-bb-{}", context.suffix);
  fs::write(
    cargo_home.join("config.toml"),
    format!(
      "[registries.gity]\nindex = \"sparse+{}/projects/{}/packages/cargo/index/\"\ncredential-provider = \"cargo:token\"\n[registry]\nglobal-credential-providers = [\"cargo:token\"]\n",
      context.api_url, context.project_id
    ),
  )?;
  fs::write(
    publisher.join("Cargo.toml"),
    format!("[package]\nname = \"{package}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n"),
  )?;
  fs::write(
    publisher.join("src/lib.rs"),
    "pub fn marker() -> &'static str { \"cargo registry roundtrip\" }\n",
  )?;
  fs::write(
    consumer.join("Cargo.toml"),
    format!(
      "[package]\nname = \"gity-bb-consumer\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[dependencies]\n{package} = {{ version = \"0.1.0\", registry = \"gity\" }}\n"
    ),
  )?;
  fs::write(
    consumer.join("src/main.rs"),
    format!(
      "fn main() {{ assert_eq!({}::marker(), \"cargo registry roundtrip\"); println!(\"cargo registry roundtrip\"); }}\n",
      package.replace('-', "_")
    ),
  )?;

  let mut command = container(
    context,
    "rust:1.99.1-bookworm",
    &root,
    &[
      ("CARGO_HOME", "/work/cargo-home"),
      ("CARGO_REGISTRIES_GITY_TOKEN", &context.token),
    ],
  );
  command.args([
    "-ec",
    "cd /work/publisher && cargo publish --registry gity --allow-dirty --no-verify && cd /work/consumer && cargo run --quiet",
  ]);
  process::run_status(&mut command, "native Cargo publish and consumer install")?;
  reporter.pass(
    "native-cargo-publish-install",
    format!("published and consumed {package} with Cargo"),
  );
  Ok(())
}
