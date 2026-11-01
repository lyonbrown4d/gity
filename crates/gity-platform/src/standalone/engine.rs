use super::secrets::{ConfiguredSecrets, write_private};
use crate::AppState;
use anyhow::Result;
use gity_oci::MirrorCredentialProvider;
use serde_json::json;
use std::time::Duration;

pub async fn config(state: &AppState) -> Result<gity_runtime::AngosConfig> {
  let config = &state.settings.components.registry;
  let mut files = config.engine_configs.clone();
  if files.is_empty() {
    let uri: reqwest::Url = config.engine_url.parse()?;
    anyhow::ensure!(
      matches!(uri.host_str(), Some("127.0.0.1" | "localhost")),
      "standalone engine must bind loopback"
    );
    let mut document = json!({"server":{"bind_address":"127.0.0.1","port":uri.port_or_known_default().unwrap_or(18000)},
      "global":{"access_policy":{"default":"allow"}}});
    if state.settings.storage.driver == "s3" {
      let store = &state.settings.storage;
      document["blob_store"] = json!({"s3":{"endpoint":store.s3_endpoint,"bucket":store.s3_bucket,"region":store.s3_region,
        "access_key_id":store.s3_access_key,"secret_key":store.s3_secret_key,"key_prefix":"oci"}});
    } else {
      let root = state.settings.storage_root.join("oci");
      tokio::fs::create_dir_all(&root).await?;
      document["blob_store"] = json!({"fs":{"root_dir":root}});
    }
    for (alias, source) in &config.mirrors {
      anyhow::ensure!(
        !alias.is_empty()
          && alias
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'),
        "invalid mirror alias"
      );
      let mut upstream = json!({"url":format!("https://{}",source.host)});
      if let Some(reference) = &source.credential_secret_ref {
        let credential = ConfiguredSecrets
          .resolve_basic(reference)
          .await
          .map_err(|_| anyhow::anyhow!("mirror secret could not be resolved"))?
          .ok_or_else(|| anyhow::anyhow!("mirror secret is missing"))?;
        upstream["username"] = json!(credential.username);
        upstream["password"] = json!(credential.password);
      }
      document["repository"][format!("mirror-{alias}")] = json!({"upstream":[upstream]});
    }
    let path = state.settings.storage_root.join(".runtime/angos.toml");
    write_private(&path, toml::to_string(&document)?.as_bytes()).await?;
    files.push(path);
  }
  Ok(gity_runtime::AngosConfig {
    executable: config.engine_binary.clone(),
    config_files: files,
    health_url: format!("{}/healthz", config.engine_url.trim_end_matches('/')),
    startup_timeout: Duration::from_secs(60),
    restart_limit: 3,
    shutdown_timeout: Duration::from_secs(15),
    environment: Default::default(),
  })
}
