use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Gity native Git branch protection hook")]
struct Args {
    #[arg(long, env = "GITY_GIT__BIN", default_value = "git")]
    git_bin: PathBuf,
    #[command(subcommand)]
    command: HookCommand,
}
#[derive(Subcommand)]
enum HookCommand {
    Update {
        reference: String,
        old: String,
        new: String,
    },
}
fn main() {
    let args = Args::parse();
    let result = std::env::current_dir()
        .map_err(anyhow::Error::from)
        .and_then(|repo| match args.command {
            HookCommand::Update {
                reference,
                old,
                new,
            } => gity_git::check_update(&repo, &args.git_bin, &reference, &old, &new),
        });
    if let Err(error) = result {
        eprintln!("Gity: {error:#}");
        std::process::exit(1);
    }
}
