use crate::configuration::cfg::{
  Auth, Cache, CacheType, Config, DatabaseType, IssueAttachments, S3IssueAttachments, Storage,
};
use figment2::Figment;
use figment2::providers::{Env, Format, Serialized, Toml};
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
      cache_type: CacheType::MOKA,
      url: None,
      moka_max_entries: Some(10_000),
    });
    cache.url = Some(cache_url);
  }

  let cache_type_value = env::var("GITY_CACHE_CACHE_TYPE")
    .ok()
    .or_else(|| env::var("GITY_CACHE_TYPE").ok());
  if let Some(value) = cache_type_value
    && let Some(cache_type) = parse_cache_type(value.as_str())
  {
    let cache = cfg.cache.get_or_insert(Cache {
      cache_type: CacheType::MOKA,
      url: None,
      moka_max_entries: Some(10_000),
    });
    cache.cache_type = cache_type;
  }

  if let Ok(value) = env::var("GITY_CACHE_MOKA_MAX_ENTRIES")
    && let Ok(parsed) = value.trim().parse::<u64>()
  {
    let cache = cfg.cache.get_or_insert(Cache {
      cache_type: CacheType::MOKA,
      url: None,
      moka_max_entries: Some(10_000),
    });
    cache.moka_max_entries = Some(parsed.max(1));
  }

  let database_type_value = env::var("GITY_DATABASE_DATABASE_TYPE")
    .ok()
    .or_else(|| env::var("GITY_DATABASE_TYPE").ok());
  if let Some(value) = database_type_value
    && let Some(database_type) = parse_database_type(value.as_str())
  {
    cfg.database.database_type = database_type;
  }

  if let Ok(repo_root) = env::var("GITY_STORAGE_REPO_ROOT") {
    let storage = cfg.storage.get_or_insert(Storage {
      repo_root: ".".to_string(),
      max_file_size: None,
    });
    storage.repo_root = repo_root;
  }

  apply_issue_attachment_overrides(cfg);

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

fn apply_issue_attachment_overrides(cfg: &mut Config) {
  let provider = env::var("GITY_ISSUE_ATTACHMENTS_PROVIDER").ok();
  let max_file_size = env::var("GITY_ISSUE_ATTACHMENTS_MAX_FILE_SIZE")
    .ok()
    .and_then(|value| value.parse::<usize>().ok());

  let endpoint = env::var("GITY_ISSUE_ATTACHMENTS_S3_ENDPOINT").ok();
  let region = env::var("GITY_ISSUE_ATTACHMENTS_S3_REGION").ok();
  let bucket = env::var("GITY_ISSUE_ATTACHMENTS_S3_BUCKET").ok();
  let access_key = env::var("GITY_ISSUE_ATTACHMENTS_S3_ACCESS_KEY").ok();
  let secret_key = env::var("GITY_ISSUE_ATTACHMENTS_S3_SECRET_KEY").ok();
  let public_base_url = env::var("GITY_ISSUE_ATTACHMENTS_S3_PUBLIC_BASE_URL").ok();
  let force_path_style = env::var("GITY_ISSUE_ATTACHMENTS_S3_FORCE_PATH_STYLE")
    .ok()
    .and_then(|value| parse_bool(value.as_str()));

  let has_override = provider.is_some()
    || max_file_size.is_some()
    || endpoint.is_some()
    || region.is_some()
    || bucket.is_some()
    || access_key.is_some()
    || secret_key.is_some()
    || public_base_url.is_some()
    || force_path_style.is_some();

  if !has_override {
    return;
  }

  let issue_attachments = cfg.issue_attachments.get_or_insert(IssueAttachments {
    provider: None,
    max_file_size: None,
    s3: None,
  });

  if let Some(provider) = provider {
    issue_attachments.provider = Some(provider);
  }
  if let Some(max_file_size) = max_file_size {
    issue_attachments.max_file_size = Some(max_file_size);
  }

  let s3 = issue_attachments.s3.get_or_insert(S3IssueAttachments {
    endpoint: None,
    region: None,
    bucket: None,
    access_key: None,
    secret_key: None,
    public_base_url: None,
    force_path_style: None,
  });

  if let Some(endpoint) = endpoint {
    s3.endpoint = Some(endpoint);
  }
  if let Some(region) = region {
    s3.region = Some(region);
  }
  if let Some(bucket) = bucket {
    s3.bucket = Some(bucket);
  }
  if let Some(access_key) = access_key {
    s3.access_key = Some(access_key);
  }
  if let Some(secret_key) = secret_key {
    s3.secret_key = Some(secret_key);
  }
  if let Some(public_base_url) = public_base_url {
    s3.public_base_url = Some(public_base_url);
  }
  if let Some(force_path_style) = force_path_style {
    s3.force_path_style = Some(force_path_style);
  }
}

fn parse_cache_type(value: &str) -> Option<CacheType> {
  match value.trim().to_ascii_uppercase().as_str() {
    "REDIS" => Some(CacheType::REDIS),
    "MOKA" | "MEMORY" => Some(CacheType::MOKA),
    _ => None,
  }
}

fn parse_database_type(value: &str) -> Option<DatabaseType> {
  match value.trim().to_ascii_uppercase().as_str() {
    "SQLITE" => Some(DatabaseType::SQLITE),
    "MYSQL" => Some(DatabaseType::MYSQL),
    "POSTGRES" | "POSTGRESQL" => Some(DatabaseType::POSTGRES),
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

fn parse_bool(value: &str) -> Option<bool> {
  match value.trim().to_ascii_lowercase().as_str() {
    "1" | "true" | "yes" | "on" => Some(true),
    "0" | "false" | "no" | "off" => Some(false),
    _ => None,
  }
}
