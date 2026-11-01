use super::super::{Reporter, process};
use super::{Clients, container};
use anyhow::Result;
use std::fs;

pub(super) fn run(context: &Clients, reporter: &mut Reporter) -> Result<()> {
  let root = context.work.join("pypi");
  let publisher = root.join("publisher");
  let package = format!("gitybb_{}", context.suffix);
  fs::create_dir_all(publisher.join(&package))?;
  let url = format!(
    "{}/projects/{}/packages/pypi/legacy/",
    context.api_url, context.project_id
  );
  let index = format!(
    "{}/projects/{}/packages/pypi/simple/",
    context.api_url, context.project_id
  );
  fs::write(
    publisher.join("setup.py"),
    format!(
      "from setuptools import setup\nsetup(name='{package}', version='1.0.0', packages=['{package}'])\n"
    ),
  )?;
  fs::write(
    publisher.join(&package).join("__init__.py"),
    "def marker():\n    return 'pypi registry roundtrip'\n",
  )?;
  let host = context
    .api_url
    .trim_start_matches("http://")
    .split('/')
    .next()
    .unwrap_or_default();
  let mut command = container(
    context,
    "python:3.13.2-bookworm",
    &root,
    &[
      ("PYPI_UPLOAD_URL", &url),
      ("PYPI_INDEX_URL", &index),
      ("GITY_TOKEN", &context.token),
      ("GITY_PYPI_HOST", host),
    ],
  );
  command.args([
    "-ec",
    &format!(
      "python -m pip install --no-cache-dir build twine >/dev/null && cd /work/publisher && python -m build --wheel >/dev/null && python -m twine upload --non-interactive --username token --password \"$GITY_TOKEN\" --repository-url \"$PYPI_UPLOAD_URL\" dist/* && python -m pip install --no-cache-dir --index-url \"http://token:$GITY_TOKEN@${{PYPI_INDEX_URL#http://}}\" --trusted-host \"$GITY_PYPI_HOST\" {package}==1.0.0 >/dev/null && python -c \"import {package}; assert {package}.marker() == 'pypi registry roundtrip'\""
    ),
  ]);
  process::run_status(&mut command, "native PyPI upload and pip install")?;
  reporter.pass(
    "native-pypi-upload-install",
    format!("uploaded and installed {package} with Twine and pip"),
  );
  Ok(())
}
