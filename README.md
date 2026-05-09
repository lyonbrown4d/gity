# Gity

Gity is a lightweight Git hosting platform moving to a Go-first backend with a TypeScript frontend.

This repository now uses a single Go module for the backend runtime and keeps the frontend in the existing TypeScript stack.

## Current Backend Baseline

- Go backend runtime with `arcgolabs/dix`
- Typed HTTP surface with `arcgolabs/httpx` + Fiber + Swagger/OpenAPI
- Authentication foundation through `arcgolabs/authx`
- Configuration loading through `arcgolabs/configx`
- Logging through `arcgolabs/logx`
- Application metadata/version through `internal/debug` and injected `dix.AppMeta`
- Database foundation through `arcgolabs/dbx` repository mode
- Snowflake `int64` IDs generated through `dbx`
- Dedicated migration runtime with versioned schema tracking in `schema_migrations`
- Git process boundary through native `git` subprocesses
- Git read model through `go-git`
- Current business chains online:
  - `organization -> project`
  - `project -> issue -> comment -> attachment`

The old Rust backend has been removed so the repository can move forward on one backend stack.

## Repository Layout

| Path | Responsibility |
| --- | --- |
| `cmd/server` | API server entrypoint |
| `cmd/migration` | database migration entrypoint |
| `cmd/worker` | background worker entrypoint |
| `cmd/serverapp` | reusable server `dix` application composition |
| `cmd/migrationapp` | reusable migration `dix` application composition |
| `cmd/workerapp` | reusable worker `dix` application composition |
| `cmd/standalone` | single-process migration + server + worker entrypoint using `dix` subapps |
| `internal/config` | runtime settings and config loading |
| `internal/debug` | build metadata and dix app meta provider |
| `internal/domain` | domain models and events |
| `internal/application` | application services and ports |
| `internal/infrastructure` | auth, db, persistence, storage, logging, worker, and git adapters |
| `internal/interfaces/http_server` | `httpx` + Fiber server bootstrap and lifecycle |
| `internal/interfaces/http` | HTTP endpoint modules |
| `web` | frontend app |

## Quick Start

### 1. Prerequisites

- Go 1.26+
- Git
- Docker Desktop or a compatible Docker runtime
- Node.js + pnpm for the frontend

### 2. Configure environment

```bash
cp .env.example .env
```

The current backend defaults to sqlite for the rewrite bootstrap, so Docker is optional for now.
`GITY_DATABASE__NODE_ID` controls the Snowflake node id used by `dbx`.
`GITY_STORAGE__ROOT` controls where issue attachments are stored on local disk.

### 3. Run database migrations

Separate server and worker processes do not manage schema. Run migrations explicitly before starting them:

```bash
go run ./cmd/migration
```

For local single-process usage, `cmd/standalone` runs migration first and then starts server and worker:

```bash
go run ./cmd/standalone
```

### 4. Run the backend

```bash
go run ./cmd/server
```

Current backend defaults:

- API: `http://localhost:8080/api`
- Health: `http://localhost:8080/api/health`
- Swagger UI: `http://localhost:8080/docs`
- OpenAPI: `http://localhost:8080/openapi.json`
- Database: local sqlite file from `GITY_DATABASE__DSN`

### 5. Run the frontend

```bash
cp web/.env.example web/.env
pnpm -C web install
pnpm -C web dev
```

## Current API Scaffold

Base URL: `http://localhost:8080/api`

System:
- `GET /health`
- `GET /v1/rewrite/info`

Users:
- `GET /v1/users`
- `GET /v1/users/{id}`
- `POST /v1/users`
- `GET /v1/users/{id}/tokens`
- `POST /v1/users/{id}/tokens`

Organizations:
- `GET /v1/orgs`
- `GET /v1/orgs/{id}`
- `POST /v1/orgs`
- `GET /v1/orgs/{id}/members`
- `POST /v1/orgs/{id}/members`

Projects:
- `GET /v1/projects`
- `GET /v1/projects/{id}`
- `POST /v1/projects`
- `GET /v1/projects/{id}/repository/branches`
- `GET /v1/projects/{id}/repository/commits`
- `GET /v1/projects/{id}/repository/tree`
- `GET /v1/projects/{id}/repository/blob`
- `GET /v1/projects/{id}/repository/readme`

Issues:
- `GET /v1/projects/{id}/issues`
- `GET /v1/projects/{id}/issues/{issue_iid}`
- `POST /v1/projects/{id}/issues`
- `PATCH /v1/projects/{id}/issues/{issue_iid}`
- `GET /v1/projects/{id}/issues/{issue_iid}/comments`
- `POST /v1/projects/{id}/issues/{issue_iid}/comments`
- `GET /v1/projects/{id}/issues/{issue_iid}/attachments`
- `POST /v1/projects/{id}/issues/{issue_iid}/attachments`
- `GET /v1/projects/{id}/issues/{issue_iid}/attachments/{attachment_id}`

Git Smart HTTP:
- `/<organization>/<project>.git/info/refs?service=git-upload-pack`
- `/<organization>/<project>.git/git-upload-pack`
- `/<organization>/<project>.git/info/refs?service=git-receive-pack`
- `/<organization>/<project>.git/git-receive-pack`

## Development Notes

Useful checks:

```bash
go test ./...
```

Architecture notes for the Go rewrite live in [docs/GO_REWRITE_ARCHITECTURE.md](docs/GO_REWRITE_ARCHITECTURE.md).
The product domain target lives in [docs/gitlab-like-domain-model.md](docs/gitlab-like-domain-model.md).

## Roadmap

Detailed planning lives in [ROADMAP.md](ROADMAP.md).
