use sadi::{Injector, Module, Provider, Shared};
use tracing::info;
use crate::configuration;
use crate::configuration::cfg::Config;

pub mod cfg;
pub mod loader;

pub struct ConfigurationModule;

impl Module for ConfigurationModule {
    fn providers(&self, injector: &Injector) {
        injector.provide::<Config>(Provider::root(|_injector| {
            dotenvy::dotenv().ok();
            let cfg = loader::load();
            let cfg_json = serde_json::to_string_pretty(&cfg).unwrap();
            info!("Loaded configuration:\n{}", cfg_json);
            <Shared<Config>>::from(cfg)
        }));
    }
}
