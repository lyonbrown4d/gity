use crate::configuration::cfg::Config;
use tracing::info;

pub mod cfg;
pub mod loader;

pub fn load_config() -> Config {
  dotenvy::dotenv().ok();
  let cfg = loader::load();
  let cfg_json = serde_json::to_string_pretty(&cfg).expect("Failed to serialize config");
  info!("Loaded configuration:\n{}", cfg_json);
  cfg
}
