use crate::configuration::cfg::Config;
use figment::providers::{Env, Format, Serialized, Toml};
use figment::Figment;

pub fn load() -> Config {
  Figment::new()
    .merge(Serialized::defaults(Config::default()))
    .merge(Toml::file("gity.toml").nested())
    .merge(Env::prefixed("GITY_").split("_"))
    .extract()
    .expect("Failed to load configuration")
}
