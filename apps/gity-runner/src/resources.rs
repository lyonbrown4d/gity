use anyhow::{Result, bail};
/// Runtime isolation is applied independently of untrusted pipeline YAML.
#[derive(Clone, clap::Args, serde::Serialize, serde::Deserialize)]
pub struct Resources {
  #[arg(long, env = "GITY_RUNNER__MEMORY_MB", default_value_t = 1024)]
  pub memory_mb: u32,
  #[arg(long, env = "GITY_RUNNER__CPUS", default_value_t = 2)]
  pub cpus: u32,
  #[arg(long, env = "GITY_RUNNER__PIDS_LIMIT", default_value_t = 256)]
  pub pids_limit: u32,
}
impl Default for Resources {
  fn default() -> Self {
    Self {
      memory_mb: 1024,
      cpus: 2,
      pids_limit: 256,
    }
  }
}
impl Resources {
  pub fn validate(&self) -> Result<()> {
    if !(64..=262144).contains(&self.memory_mb)
      || !(1..=256).contains(&self.cpus)
      || !(16..=65536).contains(&self.pids_limit)
    {
      bail!("invalid container CPU, memory, or process limit");
    }
    Ok(())
  }
}
