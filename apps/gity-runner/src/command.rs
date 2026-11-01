use crate::executor::{Executor, Job, Options};
use anyhow::{Result, bail};
use std::path::Path;
use tokio::process::Command;
pub(crate) fn build_command(
    job: &Job,
    options: &Options,
    workspace: &Path,
    script: &Path,
    name: &str,
    temp: &Path,
) -> Result<Command> {
    let mut command = match options.executor {
        Executor::Shell => {
            let mut command = Command::new("sh");
            command.arg("-eu").arg(script);
            command
                .env_clear()
                .env(
                    "PATH",
                    std::env::var_os("PATH")
                        .unwrap_or_else(|| "/usr/local/bin:/usr/bin:/bin".into()),
                )
                .env("HOME", temp.join("home"));
            for (key, value) in &job.env {
                if !valid_key(key) || value.contains('\0') {
                    bail!("invalid CI environment variable");
                }
                command.env(key, value);
            }
            command
        }
        Executor::Container => {
            let image = if job.image.is_empty() {
                &options.image
            } else {
                &job.image
            };
            if image.is_empty() || image.starts_with('-') || image.contains('\0') {
                bail!("invalid container image");
            }
            let env_file = temp.join("environment");
            let mut content = String::new();
            for (key, value) in &job.env {
                if !valid_key(key) || value.contains(['\0', '\n', '\r']) {
                    bail!("container CI variables must be valid single-line values");
                }
                content.push_str(key);
                content.push('=');
                content.push_str(value);
                content.push('\n');
            }
            std::fs::write(&env_file, content)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&env_file, std::fs::Permissions::from_mode(0o600))?;
            }
            let mut command = Command::new(&options.container_bin);
            #[cfg(unix)]
            {
                let identity = format!("{}:{}", unsafe { libc::getuid() }, unsafe {
                    libc::getgid()
                });
                command.args(["run", "--user", &identity]);
            }
            #[cfg(not(unix))]
            command.arg("run");
            options.resources.validate()?;
            command.args([
                "--memory",
                &format!("{}m", options.resources.memory_mb),
                "--cpus",
                &options.resources.cpus.to_string(),
                "--pids-limit",
                &options.resources.pids_limit.to_string(),
            ]);
            command
                .args([
                    "--rm",
                    "--name",
                    name,
                    "--cap-drop",
                    "ALL",
                    "--security-opt",
                    "no-new-privileges",
                    "--workdir",
                    "/workspace",
                    "--env-file",
                ])
                .arg(&env_file)
                .arg("--volume")
                .arg(format!(
                    "{}:/workspace",
                    workspace.canonicalize()?.display()
                ))
                .arg("--volume")
                .arg(format!(
                    "{}:/gity-script.sh:ro",
                    script.canonicalize()?.display()
                ))
                .arg(image)
                .args(["sh", "-eu", "/gity-script.sh"]);
            command
        }
    };
    command.current_dir(workspace);
    Ok(command)
}
fn valid_key(key: &str) -> bool {
    let mut bytes = key.bytes();
    bytes
        .next()
        .is_some_and(|b| b.is_ascii_uppercase() || b == b'_')
        && bytes.all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
}

#[cfg(test)]
#[path = "executor_tests.rs"]
mod tests;
