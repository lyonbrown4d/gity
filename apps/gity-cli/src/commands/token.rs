//! `gity token` — manage personal access tokens used by the CLI and CI.
use crate::client::{Client, Options};
use crate::format;
use anyhow::Result;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand)]
pub enum TokenCommand {
  /// List personal tokens (hashes are never returned)
  List,
  /// Mint a token; the raw value is printed once and cannot be recovered
  Create {
    #[arg(long)]
    name: String,
  },
  /// Revoke a token by identifier
  Revoke { id: i64 },
}

pub async fn run(command: TokenCommand, options: &Options) -> Result<()> {
  let client = Client::from_options(options)?;
  match command {
    TokenCommand::List => list(&client, options).await,
    TokenCommand::Create { name } => create(&client, name).await,
    TokenCommand::Revoke { id } => revoke(&client, id).await,
  }
}

/// The API is addressed by user id; resolve `me` once per command.
async fn me(client: &Client) -> Result<String> {
  let user = client.get("/users/me").await?;
  user["id"]
    .as_str()
    .map(str::to_owned)
    .ok_or_else(|| anyhow::anyhow!("could not resolve the authenticated user id"))
}

async fn list(client: &Client, options: &Options) -> Result<()> {
  let user = me(client).await?;
  let tokens = client.get(&format!("/users/{user}/tokens")).await?;
  if options.json {
    format::print_json(&tokens);
    return Ok(());
  }
  let rows = tokens
    .as_array()
    .into_iter()
    .flatten()
    .map(|token| {
      vec![
        format::text(token, "id"),
        format::text(token, "name"),
        format::text(token, "created_at"),
      ]
    })
    .collect::<Vec<_>>();
  format::table(&["ID", "NAME", "CREATED"], &rows);
  Ok(())
}

async fn create(client: &Client, name: String) -> Result<()> {
  let user = me(client).await?;
  let created = client
    .post(&format!("/users/{user}/tokens"), json!({ "name": name }))
    .await?;
  println!(
    "{}",
    format::paint("token created — it will not be shown again", "33")
  );
  println!("{}", format::text(&created, "token"));
  println!("export GITY_TOKEN={}", format::text(&created, "token"));
  Ok(())
}

async fn revoke(client: &Client, id: i64) -> Result<()> {
  let user = me(client).await?;
  client.delete(&format!("/users/{user}/tokens/{id}")).await?;
  println!("revoked token {id}");
  Ok(())
}
