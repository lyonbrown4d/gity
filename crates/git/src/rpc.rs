use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
  #[error("unsupported git service: {0}")]
  UnsupportedService(String),
  #[error("failed to spawn git process: {0}")]
  Spawn(String),
  #[error("failed to write request body: {0}")]
  WriteRequest(String),
  #[error("failed to wait git process: {0}")]
  Wait(String),
  #[error("git process exited with status {status}: {stderr}")]
  ProcessFailed { status: String, stderr: String },
}

pub fn run_stateless_rpc(
  repo_path: &Path,
  service: &str,
  request_body: &[u8],
) -> Result<Vec<u8>, RpcError> {
  if !matches!(service, "upload-pack" | "receive-pack") {
    return Err(RpcError::UnsupportedService(service.to_string()));
  }

  let mut child = Command::new("git")
    .arg("-C")
    .arg(repo_path)
    .arg(service)
    .arg("--stateless-rpc")
    .arg(".")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|err| RpcError::Spawn(err.to_string()))?;

  if let Some(mut stdin) = child.stdin.take() {
    stdin
      .write_all(request_body)
      .map_err(|err| RpcError::WriteRequest(err.to_string()))?;
  }

  let output = child
    .wait_with_output()
    .map_err(|err| RpcError::Wait(err.to_string()))?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    return Err(RpcError::ProcessFailed {
      status: output.status.to_string(),
      stderr: if stderr.is_empty() {
        "no stderr output".to_string()
      } else {
        stderr
      },
    });
  }

  Ok(output.stdout)
}
