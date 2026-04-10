# Gity

Gity is a lightweight Git hosting platform moving to a Go-first backend with a TypeScript frontend.

This repository now uses a single Go module for the backend runtime and keeps the frontend in the existing TypeScript stack.

## Current Backend Baseline

- Go backend runtime with `arcgo/dix`
- Typed HTTP surface with `arcgo/httpx` + Swagger/OpenAPI
- Authentication foundation through `arcgo/authx`
- Configuration loading through `arcgo/configx`
- Logging through `arcgo/logx`
- Database foundation through `arcgo/dbx`
- Git process boundary through native `git` subprocesses
- First domain chain online: `namespace -> project`

The old Rust backend has been removed so the repository can move forward on one backend stack.

## Repository Layout

| Path | Responsibility |
| --- | --- |
| `cmd/server` | API server entrypoint |
| `cmd/worker` | background worker entrypoint |
| `internal/app` | `dix` composition root |
| `internal/config` | runtime settings and config loading |
| `internal/entity` | typed dbx entities and schema definitions |
| `internal/repository` | persistence access on top of `dbx` |
| `internal/service` | application services |
| `internal/http` | `httpx` server bootstrap and lifecycle |
| `internal/endpoint` | HTTP route registration |
| `internal/platform` | auth, db, logging, and git infrastructure |
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

### 3. Run the backend

```bash
go run ./cmd/server
```

Current backend defaults:

- API: `http://localhost:8080/api`
- Health: `http://localhost:8080/api/health`
- Swagger UI: `http://localhost:8080/docs`
- OpenAPI: `http://localhost:8080/openapi.json`
- Database: local sqlite file from `GITY_DATABASE_DSN`

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

Namespaces:
- `GET /v1/namespaces`
- `GET /v1/namespaces/{id}`
- `POST /v1/namespaces`

Projects:
- `GET /v1/projects`
- `GET /v1/projects/{id}`
- `POST /v1/projects`

## Development Notes

Useful checks:

```bash
go test ./...
```

Architecture notes for the Go rewrite live in [docs/GO_REWRITE_ARCHITECTURE.md](docs/GO_REWRITE_ARCHITECTURE.md).

## Roadmap

Detailed planning lives in [ROADMAP.md](ROADMAP.md).
