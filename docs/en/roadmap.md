<div lang="en">

[简体中文](../zh/roadmap.md) · [English](roadmap.md)

# Current Product Scope

## Product Direction

Gity provides Git hosting and collaboration for small teams and self-hosted deployments. Users can complete core workflows through the Web interface and standard Git tools.

The current product is Web-based. The server uses Rust, Axum, Tokio, and SeaORM; the Web app uses Rust, Leptos, and WebAssembly; FluxDI assembles the server. Runner, Git hook, and search tools are also Rust applications. A desktop application is not in scope.

CI configuration uses typed and validated YAML, with `.gity-ci.yml` as the default repository file. The current database schema is defined by versioned Rust migrations for SQLite, PostgreSQL, and MySQL. The initial baseline requires an empty database; see [Database migrations](database-migrations.md) for upgrade prerequisites.

## Core Workflows

- **Repository**: Create or import a project, configure credentials, clone and push, then browse the README, code, branches, and commits.
- **Collaboration**: Create issues, assign and label work, create branches and merge requests, review and approve changes, merge, and inspect audit records.
- **CI**: Edit and validate YAML, create pipelines, run them on project-scoped Runners, inspect logs and artifacts, and cancel or retry jobs.
- **Packages and releases**: Publish and view packages, versions, Git tags, releases, and associated assets.
- **Operations**: Manage users, projects, Runners, storage, and audits; verify upgrades and data recovery.

## Current Boundaries

- The server enforces permissions. Project roles are `guest`, `reporter`, `developer`, `maintainer`, and `owner`; a project user receives the higher of their direct project role and organization role. Project tokens are limited to their own scopes and cannot borrow the creator’s permissions.
- Bare Git repositories are stored under names derived from immutable project IDs, such as `{project_id}.git`. Organization and project paths can change; renaming updates the URL’s `full_path` without moving the physical repository.
- Runners execute scripts from project configuration and should be used with trusted projects and trusted Runner hosts. Secret redaction is best effort and is not a security boundary against disclosure.
- Package registry protocol coverage is limited. CODEOWNERS and approval rules do not implement the full GitLab semantics. Code-search indexing and incremental updates also have limitations.
- High availability, multi-node deployment, disaster recovery, and backup/restore tooling are not currently accepted capabilities. Consult implementation and verification records for the supported SSH Git and Runner isolation behavior.

## Current Verification Status

As of 2026-11-01, local checks covered PostgreSQL 18 and SeaweedFS 4.51 integration, Git/SSH operations between two clones, Runner container jobs and artifacts, and browser checks for Releases/Runners and representative pages. See the [verification status](verification.md) for the checked scope and outstanding items.

Before release, main-branch changes must pass Linux x86 GitHub CI, and the tag-triggered release workflow must pass its builds and checks for the release architectures. The complete production backup and restore procedure also requires verification. Other architectures and the PowerShell release scripts have not been tested.

Load testing was cancelled. No throughput or performance-curve measurements are available. Verification records must distinguish source checks, compilation, executed tests, and browser or deployment acceptance; success in one area does not establish overall release readiness.

</div>
