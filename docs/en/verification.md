<div lang="en">

[简体中文](../zh/verification.md) · [English](verification.md)

# Current verification status

This page separates current source verification from retained image evidence. Administrator, GraphQL, MCP, and OAuth changes passed source regression. The current source was not rebuilt for container acceptance, so retained image results do not verify these capabilities in an image.

## Administrator and multi-protocol API checks

Default workspace checks, strict Clippy, Web WASM checks, release API checks, source policy, and the bilingual mdBook build were executed. Default-route tests confirm 404 for documentation and Swagger assets while LFS remains available. Enabling `api-docs` in a release build is rejected by the compiler.

Real application-service tests cover GraphQL project reads, MCP discovery/invocation, project isolation, OAuth resource/scopes, rejection after revocation, PKCE codes, browser consent, refresh rotation, and administrator reauthentication. The refresh race uses two independent database connections, passed 20 repeated runs, and checks descendants inserted after grant revocation. This is a concurrency regression, not a load test.

See the [source validation evidence](../evidence/api-admin-validation.json). Debian/Alpine image acceptance and PostgreSQL/MySQL container checks were not rerun; those entries below are retained records from their listed logs.

## Completed source-test runs

| Check | Current completed run | Evidence |
| --- | --- | --- |
| Workspace tests | **379 passed, 0 failed, 13 ignored by default** | `target/verification/api-admin/workspace-tests.log`; totals from all test summaries. |
| PostgreSQL 18 business regression | **81 passed** | PostgreSQL child-test section of `target/production-blackbox/database-regression.log`. |
| MySQL 8.4 business regression | **81 passed** | `target/production-blackbox/mysql-regression-final.log`. |
| PostgreSQL/MySQL migration schema contracts (retained run) | **2 passed** | `target/production-blackbox/migration-databases.log`. |
| PostgreSQL/MySQL merge-lock checks | **7 passed each** | `cargo test -p gity-test-support --test merge_guard_databases -- --ignored --test-threads=1`; isolated Testcontainers covering recovery, abort, and REST reference isolation. |
| GNU tar metadata restoration check | **1 passed** | `target/production-blackbox/backup-metadata-tests.log`. |
| Real Docker OCI authentication handshake check | **1 passed** | Separate execution of `gateway::docker_handshake_tests::real_docker_requires_authenticated_ping_for_login_and_challenge_for_push`; tool-session evidence, no separately archived log. |

The workspace collection includes typed persistence, authorization isolation, business/audit transactions, concurrent IID allocation, pipeline deduplication, Runner leases, LFS, Git HTTP/SSH, personal SSH keys, MR merge recovery, backup manifests, and load-tool unit/process checks. The final ordinary workspace rerun passed; complete container acceptance still requires each black-box combination.

The 81-test business collections ran earlier in this round. The final database merge-lock checks are listed separately; the entire business collection was not rerun after those final additions.

The database collections use `GITY_TEST_DATABASE_URL` for fixtures that support it. Transport tests that create SQLite databases and pure logic tests retain their own resources. “81 passed” does not mean every test changed to PostgreSQL/MySQL. Each container and database is disposable and does not reuse retained environments. Run database regression with:

```sh
cargo test -p gity-test-support --test database_regression --locked -- --ignored --test-threads=1
```

The 13 default-ignored checks comprise two entity container contracts, two migration container contracts, two business database entry points, two merge-lock database entry points, and one each for OCI Docker handshake, Docker Hub upstream resolution, container Runner, GNU tar metadata, and explicit local-bundle validation. Migration, business database, merge-lock database, OCI handshake, metadata, and explicit local-bundle checks have separate execution evidence; this does not change the workspace log's ignored count. Independent execution results for entity contracts, upstream resolution, and container Runner are not confirmed on this page and cannot be inferred from other scenarios.

Run the OCI handshake check separately with `cargo test -p gity-oci --lib real_docker_requires_authenticated_ping --locked -- --ignored --nocapture`. This run used Docker 29.9.2 and checked authenticated probing, wrong-login denial, successful login, and a real push.

The GNU tar check covers numeric UID/GID, permissions, extended attributes, ACLs, file capabilities, SGID, and whiteout devices. It verifies archive metadata handling and does not replace complete recovery of the database, Git, object storage, and secret configuration.

## Source, compilation, and documentation checks

- Latest formatting, source policy, third-party asset checks, and `git diff --check` passed.
- Default workspace checks, final Clippy, Web WASM, mdBook build, and bilingual link checks passed.
- Check default production and test builds separately; `gity-core/test-fixtures` must not hide production dependency errors.

## Full image black-box scope

| Deployment | Base image | Current status |
| --- | --- | --- |
| Standalone | Debian | **43 checks passed for the retained image; current source not rebuilt**; see [itemized evidence](../evidence/standalone-debian.json). |
| Standalone | Alpine | Image built; full black-box acceptance not executed. |
| Distributed components | Debian | Incomplete; startup checks stopped at registry configuration validation. |
| Distributed components | Alpine | Full black-box acceptance not executed. |

The 43 Debian standalone checks cover real HTTP/SSH Git, personal/deploy key permissions, interrupted MR recovery, Runner container jobs, package protocols, OCI/mirror, cross-project isolation, failure behavior, and a complete offline backup → new-project restore → restart → new-job execution. Hosted-object checks through private Angos prove internal data integrity, not public hosted authorization while a project's policy remains mirror.

One Debian standalone container snapshot measured 74.18 MiB for the application and 282.5 MiB for DIND. This is a single observation after functional checks, not steady-state memory, RSS, or a capacity conclusion.

Retain actual image IDs, platform, checks, failure details, and cleanup results for each combination. A passing deployment or base image does not establish another combination. See [image black-box checks](deployment/images.md#image-black-box-acceptance) and [deployment recovery boundaries](production-deployment.md).

## Not executed or not covered this round

The user canceled actual load testing for this round. Git/SSH and Git REST load tools remain available, but this round runs neither k6 nor real-client load samples and provides no throughput, loaded-latency, steady-state CPU, or peak-memory-under-load conclusions. Script inspection, load-tool unit tests, and timing Gauges are not actual load results.

This page provides no acceptance conclusion for arm64, multi-host networking, real production-data recovery, or lossless automatic failover. Only Debian standalone has passed complete recovery acceptance; the other combinations require independent verification.

</div>
