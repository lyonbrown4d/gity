<div lang="en">

[简体中文](../zh/verification.md) · [English](verification.md)

# Current verification status

This page distinguishes source checks, compilation, executed tests, and image acceptance. A successful build does not establish complete functional acceptance.

## Source and compilation checks

- Default workspace `cargo check --workspace --locked` passed; production dependencies do not enable `gity-core/test-fixtures`.
- Native workspace `cargo clippy --workspace --all-targets --locked -- -D warnings` passed.
- The Web `wasm32-unknown-unknown` target check passed.
- Formatting, the 300-line limit, and implementation/test separation passed. Source policy prevents platform business adapters from using generic Db CRUD, raw SQL, or direct Entity/ActiveModel operations.
- mdBook build, bilingual chapter pairing, local links and anchors, and local Mermaid asset checks passed.

Checks pass on the pinned toolchain. Cargo still reports a future Rust compatibility note for the third-party `proc-macro-error2 v2.0.1` dependency.

## Executed tests

Workspace execution recorded **259 passing tests and 8 ignored by default**. Coverage includes typed persistence, project isolation, atomic business/audit rollback, secret exclusion, concurrent IID allocation, pipeline deduplication, Runner leases, concurrent LFS uploads, and real Git HTTP/SSH permissions and branch protection.

The platform and server regression collection passed **71 tests each** with isolated PostgreSQL 18 and MySQL 8.4 Testcontainers. `GITY_TEST_DATABASE_URL` switches configurable database fixtures; pure logic tests and transport tests that create their own SQLite databases retain their own resources. This does not imply every test switches database backends. Containers and databases are disposable and do not reuse retained inspection data.

Run both database collections:

```sh
cargo test -p gity-test-support --test database_regression --locked -- --ignored --test-threads=1
```

The two database regression entry points ignored by default were executed separately. The other six specialized checks were not rerun for this business-persistence verification; they include entity/migration container contracts, OCI upstream resolution, and the container Runner. An ignored test is neither a pass nor a failure.

## Unverified scope

Debian/Alpine application images were not rebuilt for this verification, and the four complete standalone/distributed × Debian/Alpine image black-box combinations remain unverified. Database container regression does not replace image, OCI/mirror, or distributed failure acceptance. See [image black-box checks](deployment/images.md#image-black-box-acceptance).

There are no steady-state CPU, peak-memory, or throughput measurements. Arm64, multi-host networking, and production backup restoration remain unverified.

</div>
