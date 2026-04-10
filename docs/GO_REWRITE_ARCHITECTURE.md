# Go Rewrite Architecture

This repository is moving from the Rust workspace to a single Go module rooted at the repository root.

## Goals

- Keep one backend language across API server, worker, and future runner.
- Use `arcgo` as the infrastructure foundation.
- Keep the repository lightweight: single `go.mod`, `cmd + internal`, no multi-module workspace split.
- Preserve GitLab-like domain modeling: `namespace -> project` is the core business chain.

## Foundation Packages

- `arcgo/dix`: typed modular application runtime and lifecycle wiring.
- `arcgo/httpx`: typed HTTP server and OpenAPI surface.
- `arcgo/authx`: authentication and authorization engine.
- `arcgo/configx`: layered config loading from `.env`, environment variables, and defaults.
- `arcgo/logx`: structured logging.
- `arcgo/dbx`: schema-first ORM core and migration foundation.
- `arcgo/dbx/migrate`: migration entrypoint for later schema rollout.

## Repository Layout

- `cmd/server`: public API binary.
- `cmd/worker`: background jobs and maintenance worker binary.
- `internal/app`: `dix` composition root.
- `internal/config`: typed runtime settings.
- `internal/entity`: typed dbx entities and schema resources.
- `internal/repository/core`: startup schema bootstrap and shared persistence utilities.
- `internal/repository/namespace`: namespace persistence.
- `internal/repository/project`: project persistence.
- `internal/service/namespace`: namespace application service.
- `internal/service/project`: project application service.
- `internal/http`: `httpx` server bootstrap and lifecycle host.
- `internal/endpoint/system`: health and rewrite metadata endpoints.
- `internal/endpoint/namespace`: namespace CRUD endpoints.
- `internal/endpoint/project`: project CRUD endpoints.
- `internal/platform/auth`: `authx` runtime wrapper.
- `internal/platform/database`: `dbx` database bootstrap.
- `internal/platform/logger`: `logx` logger bootstrap.
- `internal/platform/gitexec`: native git subprocess boundary.
- `internal/platform/gittransport`: Git protocol operations built on `gitexec`.

## Current Runtime Shape

The current composition root does three things:

1. Builds config, logger, db, auth, and git runtime dependencies.
2. Runs `dbx.AutoMigrate(...)` for the first typed schemas at startup.
3. Registers `system`, `namespace`, and `project` HTTP endpoints on one `httpx` server.

## Current Domain Baseline

The first online schema set is:

- `namespaces`
- `projects`

Current rules:

- `namespace.full_path` is unique.
- `project.full_path` is unique.
- `project.namespace_id -> namespaces.id` is a foreign key with cascade delete.
- `project.full_path` is derived from `namespace.full_path + "/" + project.path_key`.

This is the first GitLab-like business chain and is intended to become the base for issues, merge requests, package registry, and Git repository provisioning.

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
6. Replace startup `AutoMigrate` bootstrap with explicit `dbx/migrate` history-managed migrations.
7. Migrate issue, package registry, LFS, and worker jobs.
