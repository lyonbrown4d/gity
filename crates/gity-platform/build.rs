fn main() {
  println!("cargo::rustc-check-cfg=cfg(gity_release)");
  println!("cargo::rerun-if-env-changed=PROFILE");
  if std::env::var("PROFILE").as_deref() == Ok("release") {
    println!("cargo::rustc-cfg=gity_release");
  }
}
