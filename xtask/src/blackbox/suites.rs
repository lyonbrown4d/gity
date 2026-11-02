use super::*;

pub(super) fn run(
  root: &Path,
  args: &BlackboxArgs,
  config: &RunConfig,
  work: &Path,
  reporter: &mut Reporter,
) -> Result<()> {
  let mut scenario = api::run(root, args, config, reporter)?;
  ci::run(root, args, config, &scenario, reporter)?;
  scenario.git_credential = Some(git::run(root, args, config, &scenario, work, reporter)?);
  git::personal::run(
    config,
    &scenario,
    scenario
      .git_credential
      .as_ref()
      .context("Git suite credential")?,
    work,
    reporter,
  )?;
  merge_recovery::run(root, args, config, &scenario, work, reporter)?;
  oci::run(config, &scenario, work, reporter)?;
  native::run(args, &scenario, work, reporter)?;
  if args.mode == Mode::Distributed {
    api::distributed(
      root,
      args,
      config,
      &scenario,
      &scenario.owner_token,
      work,
      reporter,
    )?;
  }
  // Validate live role OS/UID and sample resources before the offline backup
  // intentionally stops the entire source project.
  runtime::capture_images(root, args, reporter)?;
  backup_restore::run(root, args, config, &scenario, work, reporter)?;
  Ok(())
}
