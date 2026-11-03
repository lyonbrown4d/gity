<div lang="en">

[简体中文](../zh/functional-checks.md) · [English](functional-checks.md)

# Functional checks

Prepare an isolated environment using [Compose operations](deployment/compose.md). The following defines current coverage to check, not a claim that every case has passed.

1. Identity and access: registration, login, invalid passwords, token scopes, revocation, private project isolation.
2. Git: project creation, file commits, real HTTP/SSH clone, push and pull, key isolation and branch rules.
3. Collaboration: issues, membership, MR review/merge, permissions, failures and audit behavior.
4. CI: `.gity-ci.yml`, project Runner registration, job claiming/execution, source checkout, logs/artifacts, cancel and retry.
5. Packages: native publish/download/consumption, Generic Range, digests, idempotency and project isolation.
6. OCI: real push/pull, scopes, revocation and cross-repository mount ACLs; mirror pulls, content consistency and read-only policy.
7. Distribution: component outage/recovery, replica generations, fenced writes and controlled promotion, including isolation boundaries.
8. Web: the current React/Vite JavaScript, CSS, local editor resources, language selection, key pages and interactions.

Use `cargo xtask blackbox`; see [image checks](deployment/images.md#image-black-box-acceptance) for options and combinations. It deletes the dedicated project's volumes by default. Never use a retained environment's project name. Actual coverage is recorded in [verification status](verification.md).

</div>
