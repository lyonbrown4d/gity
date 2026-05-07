# Go Rewrite Architecture

This repository is moving from the Rust workspace to a single Go module rooted at the repository root.

## Goals

- Keep one backend language across API server, worker, and future runner.
- Use `arcgolabs` packages as the infrastructure foundation.
- Keep the repository lightweight: single `go.mod`, `cmd + internal`, no multi-module workspace split.
- Preserve GitLab-like domain modeling: `namespace -> project` is the core business chain.

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
- `internal/app`: `dix` composition root.
- Each subpackage owns its own `Module()` declaration; `internal/app` only assembles them.
- `internal/config`: typed runtime settings.
- `internal/entity`: typed dbx entities and schema resources.
- `internal/repository/core`: startup migration runner and shared persistence utilities.
- `internal/repository/namespace`: namespace persistence using `dbx` repository mode.
- `internal/repository/project`: project persistence using `dbx` repository mode.
- `internal/repository/projectissue*`: issue persistence using `dbx` repository mode.
- `internal/service/namespace`: namespace application service.
- `internal/service/project`: project application service.
- `internal/service/issue`: issue application service.
- `internal/http`: `httpx` + Fiber server bootstrap and lifecycle host.
- `internal/endpoint/system`: health and rewrite metadata endpoints.
- `internal/endpoint/namespace`: namespace CRUD endpoints.
- `internal/endpoint/project`: project CRUD and repository read endpoints.
- `internal/endpoint/issue`: issue, comment, and attachment endpoints.
- `internal/platform/auth`: `authx` runtime wrapper.
- `internal/platform/database`: `dbx` database bootstrap.
- `internal/platform/logger`: `logx` logger bootstrap.
- `internal/platform/storage`: local attachment storage.
- `internal/platform/gitexec`: native git subprocess boundary.
- `internal/platform/gittransport`: Git protocol operations built on `gitexec`.

## Current Runtime Shape

The current composition root does five things:

1. Assembles package-local `dix` modules without centralizing provider declarations in one giant runtime module.
2. Builds config, logger, db, auth, storage, and git runtime dependencies.
3. Runs versioned schema migrations tracked in `schema_migrations`.
4. Registers `system`, `user`, `namespace`, `project`, `issue`, and git transport HTTP endpoints on one Fiber-backed `httpx` server.
5. Keeps Git repository reads on `go-git` and transport on native `git`.

## Current Domain Baseline

The online schema set is:

- `users`
- `user_access_tokens`
- `namespaces`
- `namespace_members`
- `projects`
- `project_issues`
- `project_issue_comments`
- `project_issue_attachments`

Current rules:

- `namespace.full_path` is unique.
- `project.full_path` is unique.
- `project.namespace_id -> namespaces.id` is a foreign key with cascade delete.
- `project.full_path` is derived from `namespace.full_path + "/" + project.path_key`.
- `project_issue.iid` is project-local and allocated incrementally.
- `namespace.id`, `project.id`, and issue-related ids are Snowflake-generated `int64` values from `dbx`.

This is the current base for issues, package registry, LFS, and later worker jobs.

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
2. Rebuild `namespace` and `project` APIs first.
3. Add Git repository provisioning to `project create`.
4. Reintroduce Git read paths on `go-git`.
5. Reintroduce push/fetch via native git transport adapters.
6. Bring `issue`, `comment`, and `attachment` online.
7. Migrate package registry, LFS, and worker jobs.
