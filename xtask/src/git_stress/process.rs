use super::{Config, Stats};
use anyhow::{Context, Result, bail};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

#[cfg(test)]
#[path = "process_tests.rs"]
mod tests;

pub(super) fn git(
    config: &Config,
    askpass: &Askpass,
    token: &str,
    args: &[&str],
    cwd: &Path,
) -> Result<()> {
    let mut command = Command::new("git");
    command.current_dir(cwd).args(["-c", "credential.helper="]);
    if is_local(&config.base_url) || is_local(&config.project_url) {
        command.args(["-c", "http.proxy=", "-c", "https.proxy="]);
    }
    command
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "Never")
        .env("GIT_ASKPASS", &askpass.path)
        .env("GIT_ASKPASS_REQUIRE", "force")
        .env("GITY_GIT_PERF_USERNAME", &config.token_username)
        .env("GITY_GIT_PERF_TOKEN", token)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if is_local(&config.base_url) || is_local(&config.project_url) {
        let existing = std::env::var("NO_PROXY")
            .or_else(|_| std::env::var("no_proxy"))
            .unwrap_or_default();
        let merged = [
            existing,
            "127.0.0.1,localhost,::1,host.docker.internal".to_owned(),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(",");
        command.env("NO_PROXY", &merged).env("no_proxy", merged);
    }
    let mut child = ChildGroup::spawn(&mut command).context("start git client")?;
    let stdout = child.child.stdout.take().context("capture git stdout")?;
    let stderr = child.child.stderr.take().context("capture git stderr")?;
    let out_reader = thread::spawn(move || read_bounded(stdout, 64 * 1024));
    let err_reader = thread::spawn(move || read_bounded(stderr, 64 * 1024));
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(error) => {
                child.terminate_group();
                let _ = out_reader.join();
                let _ = err_reader.join();
                return Err(error).context("wait for git client");
            }
        }
        if started.elapsed() > Duration::from_millis(config.timeout_ms) {
            child.terminate_group();
            let _ = out_reader.join();
            let _ = err_reader.join();
            bail!("git operation exceeded {} ms timeout", config.timeout_ms);
        }
        thread::sleep(Duration::from_millis(25));
    };
    child.terminate_group();
    let out = out_reader.join().unwrap_or_default();
    let err = err_reader.join().unwrap_or_default();
    if status.success() {
        return Ok(());
    }
    let detail = if err.trim().is_empty() { out } else { err };
    let detail = sanitize(detail.trim(), token);
    bail!(
        "git operation failed with {status}: {}",
        detail.chars().take(2000).collect::<String>()
    )
}

fn read_bounded(mut reader: impl std::io::Read, limit: usize) -> String {
    let mut retained = Vec::with_capacity(limit);
    let mut chunk = [0_u8; 8192];
    let mut truncated = false;
    loop {
        match reader.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(read) => {
                let remaining = limit.saturating_sub(retained.len());
                let keep = read.min(remaining);
                retained.extend_from_slice(&chunk[..keep]);
                truncated |= keep < read;
            }
        }
    }
    let mut text = String::from_utf8_lossy(&retained).into_owned();
    if truncated {
        text.push_str("\n[diagnostic output truncated]");
    }
    text
}

struct ChildGroup {
    child: Child,
    process_group: i32,
    stopped: bool,
}

impl ChildGroup {
    fn spawn(command: &mut Command) -> std::io::Result<Self> {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let child = command.spawn()?;
        Ok(Self {
            process_group: child.id() as i32,
            child,
            stopped: false,
        })
    }

    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }

    fn terminate_group(&mut self) {
        if self.stopped {
            return;
        }
        #[cfg(unix)]
        unsafe {
            libc::kill(-self.process_group, libc::SIGKILL);
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.stopped = true;
    }
}

impl Drop for ChildGroup {
    fn drop(&mut self) {
        self.terminate_group();
    }
}

pub(super) fn sanitize(value: &str, token: &str) -> String {
    value.replace(token, "<token>")
}
fn is_local(raw: &str) -> bool {
    url::Url::parse(raw).ok().is_some_and(|url| {
        matches!(
            url.host_str(),
            Some("127.0.0.1" | "localhost" | "::1" | "host.docker.internal")
        )
    })
}
pub(super) fn path_string(path: &Path) -> Result<String> {
    path.to_str()
        .map(str::to_owned)
        .context("Git work path is not valid UTF-8")
}

pub(super) struct Askpass {
    path: PathBuf,
}
impl Askpass {
    pub(super) fn create(directory: &Path) -> Result<Self> {
        #[cfg(windows)]
        bail!("Git stress askpass currently requires a Unix-compatible shell");
        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            let path = directory.join("git-askpass.sh");
            fs::write(
                &path,
                "#!/bin/sh\ncase \"$1\" in\n  *sername*) printf '%s\\n' \"$GITY_GIT_PERF_USERNAME\" ;;\n  *) printf '%s\\n' \"$GITY_GIT_PERF_TOKEN\" ;;\nesac\n",
            )?;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
            Ok(Self { path })
        }
    }
}

pub(super) fn safe_remove(root: &Path, target: &Path) -> Result<()> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let target = target
        .canonicalize()
        .unwrap_or_else(|_| target.to_path_buf());
    if target == root || !target.starts_with(&root) {
        bail!("refusing to remove path outside temporary work root");
    }
    if target.exists() {
        fs::remove_dir_all(target)?;
    }
    Ok(())
}

pub(super) fn print_summary(stats: &Stats, failure_count: usize) {
    println!("\nGit client stress summary");
    for (name, values) in stats.lock().unwrap().iter() {
        let mut durations: Vec<f64> = values
            .iter()
            .map(|(duration, _)| duration.as_secs_f64() * 1000.0)
            .collect();
        durations.sort_by(f64::total_cmp);
        let avg = durations.iter().sum::<f64>() / durations.len().max(1) as f64;
        let failed = values.iter().filter(|(_, ok)| !ok).count();
        println!(
            "{name:14} count={:4} failed={failed:3} avg={avg:.1}ms p50={:.1}ms p95={:.1}ms max={:.1}ms",
            values.len(),
            percentile(&durations, 0.50),
            percentile(&durations, 0.95),
            durations.last().copied().unwrap_or(0.0)
        );
    }
    if failure_count > 0 {
        println!("\nFailures: {failure_count}");
    }
}
fn percentile(values: &[f64], quantile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let index = ((values.len() as f64 * quantile).ceil() as usize)
        .saturating_sub(1)
        .min(values.len() - 1);
    values[index]
}
