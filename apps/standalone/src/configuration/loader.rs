use crate::configuration::cfg::{Auth, Cache, CacheType, Config, Storage};
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use std::env;

pub fn load() -> Config {
  let mut cfg = Figment::new()
    .merge(Serialized::defaults(Config::default()))
    .merge(Toml::file("gity.toml").nested())
    .merge(
      Env::prefixed("GITY_")
        .ignore(&["storage_repo_root", "storage_max_file_size"])
        .split("_"),
    )
    .extract()
    .expect("Failed to load configuration");
  apply_env_overrides(&mut cfg);
  cfg
}

fn apply_env_overrides(cfg: &mut Config) {
  if let Ok(cache_url) = env::var("GITY_CACHE_URL") {
    let cache = cfg.cache.get_or_insert(Cache {
      cache_type: CacheType::MEMORY,
      url: String::new(),
    });
    cache.url = cache_url;
  }

  let cache_type_value = env::var("GITY_CACHE_CACHE_TYPE")
    .ok()
    .or_else(|| env::var("GITY_CACHE_TYPE").ok());
  if let Some(value) = cache_type_value
    && let Some(cache_type) = parse_cache_type(value.as_str())
  {
    let cache = cfg.cache.get_or_insert(Cache {
      cache_type: CacheType::MEMORY,
      url: String::new(),
    });
    cache.cache_type = cache_type;
  }

  if let Ok(repo_root) = env::var("GITY_STORAGE_REPO_ROOT") {
    let storage = cfg.storage.get_or_insert(Storage {
      repo_root: ".".to_string(),
      max_file_size: None,
    });
    storage.repo_root = repo_root;
  }

  let super_admins_value = env::var("GITY_AUTH_SUPER_ADMINS")
    .ok()
    .or_else(|| env::var("GITY_SUPER_ADMINS").ok());
  if let Some(value) = super_admins_value {
    let items = parse_csv_list(value.as_str());
    if !items.is_empty() {
      let auth = cfg.auth.get_or_insert(Auth {
        enable_jwt: None,
        jwt_secret: None,
        enable_ldap: None,
        ldap_url: None,
        super_admins: None,
        admin: None,
      });
      auth.super_admins = Some(items);
    }
  }

  if let Ok(value) = env::var("GITY_SERVER_CORS_ALLOWED_ORIGINS") {
    let items = parse_csv_list(value.as_str());
    if !items.is_empty() {
      cfg.server.cors_allowed_origins = Some(items);
    }
  }
}

fn parse_cache_type(value: &str) -> Option<CacheType> {
  match value.trim().to_ascii_uppercase().as_str() {
    "REDIS" => Some(CacheType::REDIS),
    "MEMORY" => Some(CacheType::MEMORY),
    _ => None,
  }
}

fn parse_csv_list(value: &str) -> Vec<String> {
  value
    .split(',')
    .map(str::trim)
    .filter(|item| !item.is_empty())
    .map(ToString::to_string)
    .collect()
}
