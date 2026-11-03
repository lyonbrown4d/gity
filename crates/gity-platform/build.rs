fn main() {
  println!("cargo::rustc-check-cfg=cfg(gity_release)");
  println!("cargo::rerun-if-env-changed=PROFILE");
  if std::env::var("PROFILE").as_deref() == Ok("release") {
    println!("cargo::rustc-cfg=gity_release");
  }
  // Track the directory as well as existing include_bytes! files. In particular,
  // the offline contract tool may compile before the Vite bundle exists.
  println!("cargo::rerun-if-changed=../../dist/web");
}
