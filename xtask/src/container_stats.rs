use anyhow::{Context, Result, bail};
use clap::Args;
use serde::Serialize;
use serde_json::Value;
use std::{
  fs::OpenOptions,
  io::{BufRead, BufReader, Write},
  path::PathBuf,
  process::{Command, Stdio},
  sync::mpsc::{self, RecvTimeoutError},
  thread,
  time::{Duration, SystemTime, UNIX_EPOCH},
};

const MAX_LINE_BYTES: usize = 1024 * 1024;

#[derive(Args)]
pub struct ContainerStatsArgs {
  /// Docker container ID or name (never inspected for environment variables).
  #[arg(long)]
  container: String,
  /// Sampling window in seconds.
  #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..=86400))]
  duration_seconds: u64,
  /// Label attached to every sample.
  #[arg(long, default_value = "idle")]
  label: String,
  /// New JSONL file path. Existing files are never overwritten.
  #[arg(long)]
  output: PathBuf,
}

#[derive(Serialize)]
struct Sample<'a> {
  timestamp_unix: f64,
  label: &'a str,
  stats: Value,
}

pub fn run(args: ContainerStatsArgs) -> Result<()> {
  if args.container.is_empty() || args.container.starts_with('-') {
    bail!("container must be a non-empty Docker ID or name");
  }
  let mut output = OpenOptions::new()
    .write(true)
    .create_new(true)
    .open(&args.output)
    .with_context(|| {
      format!(
        "create evidence file {} without overwriting",
        args.output.display()
      )
    })?;

  let mut child = Command::new("docker")
    .args(["stats", "--format", "{{json .}}", &args.container])
    .stdout(Stdio::piped())
    .stderr(Stdio::inherit())
    .spawn()
    .context("start Docker stats stream")?;
  let stdout = child.stdout.take().context("capture Docker stats stdout")?;
  let (tx, rx) = mpsc::sync_channel::<Result<Vec<u8>, String>>(8);
  let reader = thread::spawn(move || {
    let mut input = BufReader::new(stdout);
    loop {
      match read_bounded_line(&mut input) {
        Ok(Some(line)) => {
          if tx.send(Ok(line)).is_err() {
            break;
          }
        }
        Ok(None) => break,
        Err(error) => {
          let _ = tx.send(Err(error.to_string()));
          break;
        }
      }
    }
  });

  let deadline = std::time::Instant::now() + Duration::from_secs(args.duration_seconds);
  let mut count = 0usize;
  let mut failure = None;
  while std::time::Instant::now() < deadline {
    let remaining = deadline.saturating_duration_since(std::time::Instant::now());
    match rx.recv_timeout(remaining) {
      Ok(Ok(line)) => {
        let line = strip_terminal_controls(&line);
        if line.iter().all(u8::is_ascii_whitespace) {
          continue;
        }
        let parsed: Value = match serde_json::from_slice(&line) {
          Ok(value) => value,
          Err(error) => {
            failure = Some(format!("invalid Docker stats JSON: {error}"));
            break;
          }
        };
        let sample = Sample {
          timestamp_unix: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64(),
          label: &args.label,
          stats: parsed,
        };
        if let Err(error) = serde_json::to_writer(&mut output, &sample)
          .and_then(|_| output.write_all(b"\n").map_err(serde_json::Error::io))
        {
          failure = Some(format!("write sample: {error}"));
          break;
        }
        count += 1;
      }
      Ok(Err(error)) => {
        failure = Some(error);
        break;
      }
      Err(RecvTimeoutError::Timeout) => break,
      Err(RecvTimeoutError::Disconnected) => break,
    }
  }

  // End exactly this Docker CLI process and reap it before returning.
  let _ = child.kill();
  let status = child.wait().context("wait for Docker stats process")?;
  let _ = reader.join();
  output.flush().context("flush evidence file")?;
  if let Some(error) = failure {
    bail!("{error}");
  }
  if count == 0 {
    bail!(
      "Docker stats produced no samples (CLI status: {status}); evidence file: {}",
      args.output.display()
    );
  }
  println!(
    "Wrote {count} Docker stats samples to {}",
    args.output.display()
  );
  Ok(())
}

fn strip_terminal_controls(input: &[u8]) -> Vec<u8> {
  let mut output = Vec::with_capacity(input.len());
  let mut index = 0;
  while index < input.len() {
    if input[index] == 0x1b && input.get(index + 1) == Some(&b'[') {
      index += 2;
      while index < input.len() {
        let byte = input[index];
        index += 1;
        if (0x40..=0x7e).contains(&byte) {
          break;
        }
      }
    } else if input[index] == b'\r' {
      index += 1;
    } else {
      output.push(input[index]);
      index += 1;
    }
  }
  output
}

fn read_bounded_line(reader: &mut impl BufRead) -> std::io::Result<Option<Vec<u8>>> {
  let mut line = Vec::new();
  loop {
    let available = reader.fill_buf()?;
    if available.is_empty() {
      return if line.is_empty() {
        Ok(None)
      } else {
        Ok(Some(line))
      };
    }
    let take = available
      .iter()
      .position(|byte| *byte == b'\n')
      .map_or(available.len(), |index| index + 1);
    if line.len() + take > MAX_LINE_BYTES {
      return Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Docker stats line exceeds 1 MiB",
      ));
    }
    let has_newline = available[take - 1] == b'\n';
    line.extend_from_slice(&available[..take]);
    reader.consume(take);
    if has_newline {
      return Ok(Some(line));
    }
  }
}
