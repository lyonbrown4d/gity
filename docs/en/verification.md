<div lang="en">

[简体中文](../zh/verification.md) · [English](verification.md)

# Current verification status

This page distinguishes implementation, compilation, executed tests, and image acceptance. Passing static checks does not establish complete runtime acceptance.

## Static checks

- Native workspace and WASM Clippy passed with the pinned Rust toolchain.
- Formatting, source size/test-separation rules, and third-party editor asset checks passed.
- Standalone/distributed Compose parsing passed for both environment templates; deployment shell syntax checks passed.
- The documentation update builds the book and runs targeted Clippy for its tooling. Generated pages are checked for chapter coverage, local links, anchors, and diagram resources.

Cargo reports a future Rust compatibility note for `proc-macro-error2 v2.0.1`; the pinned toolchain passes current checks.

## Runtime coverage

Workspace execution recorded 236 passing tests and 4 ignored by default. PostgreSQL, MySQL, OCI upstream resolution, and container Runner integration checks were also run explicitly and passed.

All eight Debian roles have build records. Alpine builds exist for standalone, git-node, gateway, registry, git, api, and worker; the independent Runner Alpine image has not completed this round of building.

The four complete standalone/distributed × Debian/Alpine black-box combinations have not completed. Full image-level SSH/HTTP Git, OCI/mirror, and distributed failure flows require runtime confirmation. See [image black-box checks](deployment/images.md#image-black-box-acceptance).

There are no steady-state CPU, peak-memory, or throughput conclusions. Arm64, multi-host networking, and production backup restoration are unverified. Delivery follows the authorized static-check scope.

</div>
