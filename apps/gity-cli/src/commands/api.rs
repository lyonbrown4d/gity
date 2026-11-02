//! `gity api` — raw REST access for endpoints the CLI does not model yet.
use crate::client::{Client, Options};
use crate::format;
use anyhow::Result;
use clap::{Args, ValueEnum};
use serde_json::{Map, Value};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Method {
  Get,
  Post,
  Patch,
  Put,
  Delete,
}

#[derive(Args)]
pub struct ApiArgs {
  /// Path relative to /api/v1, for example /projects/1/issues
  path: String,
  #[arg(short = 'X', long, value_enum, default_value = "get")]
  method: Method,
  /// Request body field as key=value; repeatable, values parse as JSON when possible
  #[arg(short = 'f', long = "field")]
  fields: Vec<String>,
}

pub async fn run(args: ApiArgs, options: &Options) -> Result<()> {
  let client = Client::from_options(options)?;
  let path = if args.path.starts_with('/') {
    args.path.clone()
  } else {
    format!("/{}", args.path)
  };
  let body = body(&args.fields)?;
  let result = match args.method {
    Method::Get => client.get(&path).await?,
    Method::Post => client.post(&path, body).await?,
    Method::Patch => client.patch(&path, body).await?,
    Method::Put => client.post(&path, body).await?,
    Method::Delete => client.delete(&path).await?,
  };
  format::print_json(&result);
  Ok(())
}

fn body(fields: &[String]) -> Result<Value> {
  if fields.is_empty() {
    return Ok(Value::Object(Map::new()));
  }
  let mut map = Map::new();
  for field in fields {
    let (key, raw) = field
      .split_once('=')
      .ok_or_else(|| anyhow::anyhow!("field '{field}' must use key=value"))?;
    let value = serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_owned()));
    map.insert(key.to_owned(), value);
  }
  Ok(Value::Object(map))
}
