fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut config = tonic_prost_build::Config::new();
  config.skip_debug([
    ".gity.identity.v1.AuthorizeOciRequest",
    ".gity.identity.v1.AuthorizePackageRequest",
  ]);
  config.protoc_executable(protoc_bin_vendored::protoc_bin_path()?);
  tonic_prost_build::configure().compile_with_config(
    config,
    &["proto/git.proto", "proto/identity.proto"],
    &["proto"],
  )?;
  Ok(())
}
