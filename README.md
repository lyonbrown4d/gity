# Gity

Gity is a lightweight Git hosting platform moving to a Go-first backend with a TypeScript frontend.

This repository now uses a single Go module for the backend runtime and keeps the frontend in the existing TypeScript stack.

## Current Backend Baseline

- Go backend runtime with `arcgolabs/dix`
- Typed HTTP surface with `arcgolabs/httpx` + Fiber + Swagger/OpenAPI
- Authentication foundation through `arcgolabs/authx`
- Configuration loading through `arcgolabs/configx`
- Logging through `arcgolabs/logx`
- Database foundation through `arcgolabs/dbx` repository mode
- Snowflake `int64` IDs generated through `dbx`
- Versioned schema bootstrap through a `schema_migrations` table plus `dbx` migration execution
- Git process boundary through native `git` subprocesses
- Git read model through `go-git`
- Current business chains online:
  - `namespace -> project`
  - `project -> issue -> comment -> attachment`

The old Rust backend has been removed so the repository can move forward on one backend stack.

## Repository Layout

| Path | Responsibility |
| --- | --- |
| `cmd/server` | API server entrypoint |
| `cmd/worker` | background worker entrypoint |
| `internal/app` | thin `dix` composition root |
| `internal/config` | runtime settings and config loading |
| `internal/entity` | typed dbx entities and schema definitions |
| `internal/repository` | package-local `dbx` repository mode persistence |
| `internal/service` | application services |
| `internal/http` | `httpx` + Fiber server bootstrap and lifecycle |
| `internal/endpoint` | HTTP route registration |
| `internal/platform` | auth, db, logging, storage, and git infrastructure |
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

### 3. Run the backend

```bash
go run ./cmd/server
```

Current backend defaults:

- API: `http://localhost:8080/api`
- Health: `http://localhost:8080/api/health`
- Swagger UI: `http://localhost:8080/docs`
- OpenAPI: `http://localhost:8080/openapi.json`
- Database: local sqlite file from `GITY_DATABASE__DSN`

### 4. Run the frontend

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

Namespaces:
- `GET /v1/namespaces`
- `GET /v1/namespaces/{id}`
- `POST /v1/namespaces`
- `GET /v1/namespaces/{id}/members`
- `POST /v1/namespaces/{id}/members`

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
- `/<namespace>/<project>.git/info/refs?service=git-upload-pack`
- `/<namespace>/<project>.git/git-upload-pack`
- `/<namespace>/<project>.git/info/refs?service=git-receive-pack`
- `/<namespace>/<project>.git/git-receive-pack`

## Development Notes

Useful checks:

```bash
go test ./...
```

Architecture notes for the Go rewrite live in [docs/GO_REWRITE_ARCHITECTURE.md](docs/GO_REWRITE_ARCHITECTURE.md).

## Roadmap

Detailed planning lives in [ROADMAP.md](ROADMAP.md).
