//! `gity completion <shell>` — print a completion script for the detected shell.
use anyhow::Result;
use clap::CommandFactory;
use clap_complete::Shell;
use std::io;

pub fn run(shell: Shell) -> Result<()> {
  let mut command = crate::command();
  let name = command.get_name().to_owned();
  clap_complete::generate(shell, &mut command, name, &mut io::stdout());
  Ok(())
}

/// `gity completion` needs a `Command` value; keep the factory in one place.
#[allow(dead_code)]
fn _assert_command_factory() {
  let _ = crate::Cli::command();
}
