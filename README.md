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

The current Go rewrite is a scaffold-first baseline. The old Rust backend has been removed so the repository can move forward on one backend stack.

## Repository Layout

| Path | Responsibility |
| --- | --- |
| `cmd/server` | API server entrypoint |
| `cmd/worker` | background worker entrypoint |
| `internal/app` | `dix` composition root |
| `internal/config` | runtime settings and config loading |
| `internal/http` | `httpx` server bootstrap and lifecycle |
| `internal/endpoint` | HTTP route registration |
| `internal/platform` | auth, db, logging, and git infrastructure |
| `internal/service` | application services |
| `web` | frontend app |

## Quick Start

### 1. Prerequisites

- Go 1.26+
- Git
- Docker Desktop or a compatible Docker runtime
- Node.js + pnpm for the frontend

### 2. Start local infrastructure

```bash
docker compose up -d postgres redis minio minio-init
```

### 3. Configure environment

```bash
cp .env.example .env
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

### 5. Run the frontend

```bash
cp web/.env.example web/.env
pnpm -C web install
pnpm -C web dev
```

## Current API Scaffold

Base URL: `http://localhost:8080/api`

- `GET /health`
- `GET /v1/rewrite/info`

## Development Notes

Useful checks:

```bash
go test ./...
```

Architecture notes for the Go rewrite live in [docs/GO_REWRITE_ARCHITECTURE.md](docs/GO_REWRITE_ARCHITECTURE.md).

## Roadmap

Detailed planning lives in [ROADMAP.md](ROADMAP.md).
