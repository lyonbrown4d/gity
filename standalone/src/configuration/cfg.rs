use serde::{Deserialize, Serialize};

#[derive(Deserialize, PartialEq, Debug, Serialize, Clone, Copy)]
pub struct Server {
  pub port: usize,
  pub tls_enabled: Option<bool>,
  pub publish: Option<bool>,
}

#[derive(Deserialize, PartialEq, Debug, Serialize, Clone)]
pub struct Database {
  pub url: String,
  pub max_connections: Option<usize>,
}

#[derive(Deserialize, PartialEq, Debug, Serialize, Clone)]
pub struct Storage {
  pub repo_root: String,
  pub max_file_size: Option<usize>,
}

#[derive(Deserialize, PartialEq, Debug, Serialize, Clone)]
pub struct Auth {
  pub enable_jwt: Option<bool>,
  pub jwt_secret: Option<String>,
  pub enable_ldap: Option<bool>,
  pub ldap_url: Option<String>,
  pub admin: Option<Admin>,
}
#[derive(Deserialize, PartialEq, Debug, Serialize, Clone)]
pub struct Admin {
  pub username: Option<String>,
  pub password: Option<String>,
}

#[derive(Deserialize, PartialEq, Debug, Serialize, Clone)]
pub struct Logging {
  pub level: Option<String>,
  pub file: Option<String>,
}

#[derive(Deserialize, PartialEq, Debug, Serialize, Clone)]
pub struct Git {
  pub default_branch: Option<String>,
  pub hooks_dir: Option<String>,
}

#[derive(Deserialize, PartialEq, Debug, Serialize, Clone)]
pub struct Config {
  pub server: Server,
  pub database: Database,
  #[serde(default)]
  pub storage: Option<Storage>,
  #[serde(default)]
  pub auth: Option<Auth>,
  #[serde(default)]
  pub logging: Option<Logging>,
  #[serde(default)]
  pub git: Option<Git>,
}

impl Default for Config {
  fn default() -> Self {
    Config {
      server: Server {
        port: 8080,
        publish: Some(false),
        tls_enabled: Some(false),
      },
      database: Database {
        url: "postgres://user:pass@localhost:5432/gity".to_string(),
        max_connections: Some(10),
      },
      storage: None,
      auth: None,
      logging: None,
      git: None,
    }
  }
}
