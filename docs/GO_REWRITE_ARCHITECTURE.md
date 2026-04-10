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
- `internal/http`: `httpx` server bootstrap and lifecycle host.
- `internal/endpoint`: HTTP route registration.
- `internal/platform/auth`: `authx` runtime wrapper.
- `internal/platform/database`: `dbx` database bootstrap.
- `internal/platform/logger`: `logx` logger bootstrap.
- `internal/platform/gitexec`: native git subprocess boundary.
- `internal/platform/gittransport`: Git protocol operations built on `gitexec`.
- `internal/service`: application services.

## Git Boundary

- `go-git`: repository reads, refs, trees, blobs, history, analysis.
- `native git`: upload-pack, receive-pack, maintenance, protocol-sensitive operations.
- Business code must not call `os/exec` directly. All subprocess usage goes through `internal/platform/gitexec`.

## Migration Order

1. Replace Rust bootstrap with Go `cmd/server` and `cmd/worker` runtime.
2. Rebuild `namespace` and `project` APIs first.
3. Reintroduce Git read paths on `go-git`.
4. Reintroduce push/fetch via native git transport adapters.
5. Reintroduce schema and migrations on `dbx/dbx/migrate`.
6. Migrate issue, package registry, LFS, and worker jobs.
