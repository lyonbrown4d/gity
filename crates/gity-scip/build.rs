//! Compiles the vendored SCIP schema into prost types.
//!
//! The protobuf compiler ships with `protoc-bin-vendored`, so the build does not
//! depend on a `protoc` found on the host, matching `gity-git-wire`.
fn main() -> Result<(), Box<dyn std::error::Error>> {
  let protoc = protoc_bin_vendored::protoc_bin_path()?;
  let mut config = prost_build::Config::new();
  config.protoc_executable(protoc);
  config.compile_protos(&["proto/scip.proto"], &["proto"])?;
  Ok(())
}
