use anyhow::{Context, Result, bail};
use std::process::{Command, Output, Stdio};

pub fn run_output(command: &mut Command, name: &str) -> Result<Output> {
  let output = command
    .stdin(Stdio::null())
    .output()
    .with_context(|| format!("start {name}"))?;
  if !output.status.success() {
    bail!("{name} exited with status {}", status_code(&output));
  }
  Ok(output)
}

pub fn run_status(command: &mut Command, name: &str) -> Result<()> {
  let status = command
    .stdin(Stdio::null())
    .status()
    .with_context(|| format!("start {name}"))?;
  if !status.success() {
    bail!("{name} exited with status {}", status.code().unwrap_or(-1));
  }
  Ok(())
}

pub fn run_with_stdin(command: &mut Command, input: &[u8], name: &str) -> Result<Output> {
  let mut child = command
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .with_context(|| format!("start {name}"))?;
  if let Some(mut stdin) = child.stdin.take() {
    use std::io::Write;
    stdin
      .write_all(input)
      .context("write private child input")?;
  }
  let output = child.wait_with_output().context("wait for child process")?;
  if !output.status.success() {
    bail!("{name} exited with status {}", status_code(&output));
  }
  Ok(output)
}

fn status_code(output: &Output) -> i32 {
  output.status.code().unwrap_or(-1)
}
