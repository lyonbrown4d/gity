use super::super::{Reporter, process};
use super::{Clients, container};
use anyhow::Result;
use std::fs;

pub(super) fn run(context: &Clients, reporter: &mut Reporter) -> Result<()> {
  let root = context.work.join("npm");
  let publisher = root.join("publisher");
  let consumer = root.join("consumer");
  fs::create_dir_all(&publisher)?;
  fs::create_dir_all(&consumer)?;
  let package = format!("@gity-bb-{}/fixture", context.suffix);
  let registry = format!(
    "{}/projects/{}/packages/npm/",
    context.api_url, context.project_id
  );
  fs::write(
    root.join(".npmrc"),
    format!(
      "registry={registry}\nalways-auth=true\n{}:_authToken=${{NPM_TOKEN}}\n",
      registry.trim_start_matches("http:")
    ),
  )?;
  fs::write(
    publisher.join("package.json"),
    format!("{{\"name\":\"{package}\",\"version\":\"1.0.0\",\"main\":\"index.js\"}}\n"),
  )?;
  fs::write(
    publisher.join("index.js"),
    "module.exports = () => 'npm registry roundtrip';\n",
  )?;
  fs::write(
    consumer.join("package.json"),
    "{\"name\":\"gity-bb-npm-consumer\",\"version\":\"1.0.0\"}\n",
  )?;
  let mut command = container(
    context,
    "node:22.14.0-bookworm",
    &root,
    &[
      ("NPM_TOKEN", &context.token),
      ("NPM_REGISTRY", &registry),
      ("NPM_CONFIG_USERCONFIG", "/work/.npmrc"),
    ],
  );
  command.args([
    "-ec",
    &format!(
      "cd /work/publisher && npm publish --access public --registry \"$NPM_REGISTRY\" && cd /work/consumer && npm install --cache /work/consumer-cache --registry \"$NPM_REGISTRY\" {package}@1.0.0 && node -e \"if(require('{package}')() !== 'npm registry roundtrip') process.exit(1)\""
    ),
  ]);
  process::run_status(&mut command, "native npm publish and consumer install")?;
  reporter.pass(
    "native-npm-publish-install",
    format!("published and consumed {package} with npm"),
  );
  Ok(())
}
