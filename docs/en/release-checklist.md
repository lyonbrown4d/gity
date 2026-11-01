<div lang="en">

[简体中文](../zh/release-checklist.md) · [English](release-checklist.md)

# Release checks

Choose checks for the actual delivery scope and record outcomes and uncovered behavior in [verification status](verification.md).

## Source and build

```sh
cargo xtask source-check
cargo xtask vendor-check
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check -p gity-web --target wasm32-unknown-unknown --locked
cargo xtask docs
```

Keep application entry points, modules, database versions, configuration defaults, and deployment definitions consistent. See [image variants](deployment/images.md) for builds.

## Runtime and delivery

Check [functional behavior](functional-checks.md), protocol permissions, failures, and recovery as required. Record deployment modes and base systems separately. Retain actual image IDs, platform, configuration scope, and evidence; a successful build does not establish functional acceptance.

The current delivery commit belongs on the main branch. Prepare database, Git, and object-storage backups before deployment; restoration claims require executed drills. Documentation artifacts and application images are separate deliverables.

</div>
