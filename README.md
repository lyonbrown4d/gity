# Gity

Gity is a lightweight, Rust-first Git hosting platform focused on predictable operations, low resource usage, and incremental delivery.

This repository is a Rust workspace with API server, domain model, persistence layer, migrations, and early CI runner components.

## Project Status

- Active development
- API-first backend available
- Not production-ready yet

Implemented today:

- Auth: register, login, JWT access token
- Organizations: create org, list my orgs, add members
- Invitations: create, accept (id/token), revoke, expiry cleanup job
- Repositories: create/list repos, branch management, commit records
- Persistence: SeaORM entities + migrations for core tables

## Goals

- Small runtime footprint
- Fast startup and clear architecture
- Straightforward self-hosting
- Stable foundation for Git and collaboration workflows

## Architecture

Dependency assembly is explicit (no DI container):

- `bootstrap` initializes infrastructure (DB, Redis, config)
- `AppState` carries runtime dependencies
- HTTP handlers and services receive state explicitly

Workspace layout:

| Path | Responsibility |
| --- | --- |
| `apps/standalone` | HTTP server, routing, auth, org/repo APIs, bootstrap |
| `apps/ci-runner` | CI runner binary (early stage) |
| `crates/domain` | DTOs and shared models |
| `crates/entity` | SeaORM entity models |
| `crates/migration` | SeaORM migration definitions + migrator CLI |
| `crates/git` | Git-related integration helpers |
| `crates/repository` | Data access abstractions |

Previous crate responsibilities are preserved; only filesystem layout changed to align with `apps/` + `crates/`.

Repository modules:

| Crate | Responsibility |
| --- | --- |
| `standalone` | HTTP server, routing, auth, org/repo APIs, bootstrap |
| `entity` | SeaORM entity models |
| `migration` | SeaORM migration definitions + migrator CLI |
| `domain` | DTOs and shared models |
| `git` | Git-related integration helpers |
| `repository` | Data access abstractions (in progress) |
| `ci-runner` | CI runner binary (early stage) |

Distributed Git direction:

- See [docs/distributed-git-layer.md](docs/distributed-git-layer.md) for the phased backend plan.

## Quick Start

### 1. Prerequisites

- Rust stable toolchain
- Docker Desktop (or compatible Docker runtime)

### 2. Start infrastructure

```bash
docker compose up -d postgres redis
```

Current compose mapping:

- Postgres: `localhost:5433 -> container:5432`
- Redis: `localhost:6379`

### 3. Configure environment

```bash
cp .env.example .env
```

Important defaults:

- `GITY_SERVER_PORT=8080`
- `GITY_SERVER_CORS_ALLOWED_ORIGINS=http://localhost:5173,http://127.0.0.1:5173`
- `GITY_DATABASE_URL=postgres://root:root@localhost:5433/gity`
- `GITY_CACHE_CACHE_TYPE=REDIS`
- `GITY_CACHE_URL=redis://localhost:6379`
- `GITY_STORAGE_REPO_ROOT=./data/repos`
- `GITY_AUTH_SUPER_ADMINS=admin` (comma-separated usernames/emails)
- Super-admin matching is done against existing user `username/email` values (it does not auto-create accounts)

Recommended for non-local usage:

- `GITY_AUTH_JWT_SECRET=<strong-random-secret>`

### 4. Run the API server

```bash
cargo run -p standalone
```

On startup, migrations are applied automatically.

### 5. OpenAPI

- JSON schema: `http://localhost:8080/api-docs/openapi.json`
- Swagger UI: `http://localhost:8080/swagger-ui`

### 6. Frontend (refine + shadcn)

```bash
cp web/.env.example web/.env
pnpm -C web install
pnpm -C web dev
```

Dev defaults:

- Frontend: `http://localhost:5173`
- API base in browser: `/api/v1`
- Vite proxy target: `http://127.0.0.1:8080` (from `VITE_DEV_API_PROXY_TARGET`)

## API Snapshot

Base URL: `http://localhost:8080/api/v1`

Auth:

- `POST /auth/register`
- `POST /auth/login`

Organizations:

- `GET /orgs/me`
- `POST /orgs`
- `POST /orgs/{organization_id}/members`
- `POST /orgs/{organization_id}/invitations`
- `POST /orgs/invitations/{invitation_id}/accept`
- `POST /orgs/invitations/accept`
- `GET /orgs/invitations/accept?token=...`
- `DELETE /orgs/{organization_id}/invitations/{invitation_id}`

Repositories:

- `GET /repos?organization_id=...`
- `POST /repos`
- `GET /repos/{repo_id}/branches`
- `POST /repos/{repo_id}/branches`
- `POST /repos/{repo_id}/branches/{branch_name}/protect`
- `POST /repos/{repo_id}/branches/{branch_name}/unprotect`
- `GET /repos/{repo_id}/commits`
- `POST /repos/{repo_id}/commits`

Users:

- `GET /users` (super admin only)
- `GET /users/me`
- `PATCH /users/me`

## Development Notes

Configuration merge order:

1. Rust defaults
2. `gity.toml`
3. `GITY_*` environment variables

Useful checks:

```bash
cargo check -p standalone
cargo check -p migration
```

Git fetch/push smoke (Rust integration test):

```bash
cargo test -p standalone --test git_http_smoke git_http_smoke_push_and_branch_sync -- --ignored --nocapture
```

Notes:

- The test requires running `postgres` and `redis` (for example via `docker compose up -d postgres redis`).
- The test is marked `ignored` by default to avoid accidental CI/local failures when infra is absent.

## Roadmap

Detailed planning lives in [ROADMAP.md](ROADMAP.md).

Summary:

- Phase 1: Git data plane and smart HTTP foundations
- Phase 2: Access control and permission hardening
- Phase 3: Collaboration primitives (issues and merge requests)
- Phase 4: CI/CD minimal loop
- Phase 5: Production readiness (observability and operations)

## Contributing

Contributions are welcome.

For major changes, open an issue first to align scope, architecture, and milestone impact.
