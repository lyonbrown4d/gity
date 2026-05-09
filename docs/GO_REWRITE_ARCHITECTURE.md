# Go Rewrite Architecture

This repository is moving from the Rust workspace to a single Go module rooted at the repository root.

## Goals

- Keep one backend language across API server, worker, and future runner.
- Use `arcgolabs` packages as the infrastructure foundation.
- Keep the repository lightweight: single `go.mod`, `cmd + internal`, no multi-module workspace split.
- Preserve GitLab-like domain modeling with a product-facing `organization -> project` business chain.

## Foundation Packages

- `arcgolabs/dix`: typed modular application runtime and lifecycle wiring.
- `arcgolabs/httpx`: typed HTTP server and OpenAPI surface.
- `arcgolabs/httpx/adapter/fiber`: runtime bridge for the HTTP API host.
- `arcgolabs/authx`: authentication and authorization engine.
- `arcgolabs/configx`: layered config loading from `.env`, environment variables, and defaults.
- `arcgolabs/logx`: structured logging.
- `arcgolabs/dbx`: schema-first ORM core and migration execution foundation.

## Repository Layout

- `cmd/server`: public API binary.
- `cmd/worker`: background jobs and maintenance worker binary.
- `cmd/migration`: database migration binary.
- `cmd/serverapp`: reusable server `dix` app composition.
- `cmd/workerapp`: reusable worker `dix` app composition.
- `cmd/migrationapp`: reusable migration `dix` app composition.
- `cmd/standalone`: single-process runtime that mounts migration, server, and worker as `dix` subapps.
- Each executable command keeps its `package main` thin and delegates app construction to the matching importable app package.
- `cmd/standalone` imports `cmd/migrationapp`, `cmd/serverapp`, and `cmd/workerapp` so module composition stays reusable instead of being repeated.
- `internal/debug`: build metadata reader and `dix.AppMeta` provider.
- `internal/config`: typed runtime settings.
- `internal/domain`: domain models and events.
- `internal/infrastructure/persistence/core`: migration runner used by the migration app.
- `internal/infrastructure/persistence/organization`: organization persistence using `dbx` repository mode.
- `internal/infrastructure/persistence/project`: project persistence using `dbx` repository mode.
- `internal/infrastructure/persistence/project_issue*`: issue persistence using `dbx` repository mode.
- `internal/application/organization`: organization application service.
- `internal/application/project`: project application service.
- `internal/application/issue`: issue application service.
- `internal/interfaces/http_server`: `httpx` + Fiber server bootstrap and lifecycle host.
- `internal/interfaces/http/system`: health and rewrite metadata endpoints.
- `internal/interfaces/http/organization`: organization CRUD endpoints.
- `internal/interfaces/http/project`: project CRUD and repository read endpoints.
- `internal/interfaces/http/issue`: issue, comment, and attachment endpoints.
- `internal/infrastructure/auth`: `authx` runtime wrapper.
- `internal/infrastructure/database`: `dbx` database bootstrap.
- `internal/infrastructure/logger`: `logx` logger bootstrap.
- `internal/infrastructure/storage`: local attachment storage.
- `internal/infrastructure/git_exec`: native git subprocess boundary.
- `internal/infrastructure/git_transport`: Git protocol operations built on `git_exec`.

## Current Runtime Shape

The command composition roots do five things:

1. Assembles package-local `dix` modules through nested command modules instead of centralizing provider declarations in one giant runtime module.
2. Builds config, logger, db, auth, storage, and git runtime dependencies.
3. Keeps schema management out of server and worker apps; split-process deployments must run `cmd/migration` explicitly.
4. Registers `system`, `user`, `organization`, `project`, `issue`, and git transport HTTP endpoints on one Fiber-backed `httpx` server.
5. Uses `dix` subapps in `cmd/standalone` by reusing the migration, server, and worker app packages, so migration runs before server and worker in single-process deployments.

Application version metadata is injected through a `dix.AppMeta` provider from `internal/debug`, which reads Go build information and can be overridden through release build flags.

## Current Domain Baseline

The online schema set is:

- `users`
- `user_access_tokens`
- `organizations`
- `organization_members`
- `projects`
- `project_issues`
- `project_issue_comments`
- `project_issue_attachments`

Current rules:

- `organization.full_path` is unique.
- `organization.visibility` is normalized to `private`, `internal`, or `public`.
- `project.full_path` is unique.
- `project.organization_id -> organizations.id` is a foreign key with cascade delete.
- `project.full_path` is derived from `organization.full_path + "/" + project.path_key`.
- `project_issue.iid` is project-local and allocated incrementally.
- `organization.id`, `project.id`, and issue-related ids are Snowflake-generated `int64` values from `dbx`.

This is the current base for issues, package registry, LFS, and worker jobs.

## dbx Style

- `dbx` repository mode is the default persistence style for the backend.
- Active record mode is intentionally not used in the main application chain.
- Domain write paths stay in services; repositories stay focused on schema-backed persistence.
- The current codebase uses a small versioned migration runner built on top of `dbx` because the currently pinned `arcgolabs/dbx` version does not expose a separate `dbx/migrate` package.

## Git Boundary

- `go-git`: repository reads, refs, trees, blobs, history, analysis.
- `native git`: upload-pack, receive-pack, maintenance, protocol-sensitive operations.
- Business code must not call `os/exec` directly. All subprocess usage goes through `internal/platform/gitexec`.

## Migration Order

1. Replace Rust bootstrap with Go `cmd/server` and `cmd/worker` runtime.
2. Rebuild `organization` and `project` APIs first.
3. Add Git repository provisioning to `project create`.
4. Reintroduce Git read paths on `go-git`.
5. Reintroduce push/fetch via native git transport adapters.
6. Bring `issue`, `comment`, and `attachment` online.
7. Migrate package registry, LFS, and worker jobs.
