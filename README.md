# Gity

A lightweight, Rust-first GitLab alternative focused on low memory footprint, clear architecture, and incremental delivery.

## Project Vision

Gity aims to provide the core workflows teams need from Git hosting and collaboration platforms, without the operational overhead of a full GitLab deployment.

Primary goals:

- Small runtime footprint
- Fast startup and predictable performance
- Modular workspace architecture
- Straightforward self-hosting
- API-first evolution

## Current Status

Implemented today:

- Authentication
  - User registration
  - User login
  - JWT access token
- Organization management
  - Create organization
  - List current user's organizations
  - Add organization members (owner-only)
- Invitation workflow
  - Create invitation token + acceptance URL
  - Accept invitation by ID or token
  - Revoke invitation
  - Background expiration cleanup job (every 5 minutes)
- Repository foundation
  - Create/list repositories
  - Create/list branches
  - Protect/unprotect branch (owner-only)
  - Record/list commits
  - Protected-branch write restriction (owner-only)
- Database
  - SeaORM entities + migrations for users, organizations, memberships, invitations, repositories, branches, commits

## Why No DI Container?

This codebase currently uses explicit dependency assembly (no DI framework):

- `bootstrap` builds infra once (DB, Redis, config)
- `AppState` holds shared runtime dependencies
- handlers/services receive state explicitly

This keeps async behavior simple and avoids framework-level limitations around async provider construction.

## Workspace Layout

| Crate | Responsibility |
| --- | --- |
| `standalone` | HTTP server, routing, auth, org/repo APIs, bootstrap |
| `entity` | SeaORM entity models |
| `migration` | SeaORM migration definitions + migrator CLI |
| `domain` | Domain DTOs / shared models |
| `git` | Git-related integration helpers |
| `repository` | Data access abstractions (in progress) |
| `ci-runner` | CI runner binary (early stage) |

## Quick Start

### 1. Prerequisites

- Rust stable toolchain
- Docker (recommended for Postgres/Redis)

### 2. Start infrastructure

```bash
docker compose up -d
```

### 3. Configure environment

```bash
cp .env.example .env
```

Important defaults in `.env.example`:

- `GITY_DATABASE_URL=postgres://root:root@localhost:5432/gity`
- `GITY_SERVER_PORT=8080`

Optional but recommended:

- `GITY_AUTH_JWT_SECRET=change-me-in-production`

### 4. Run the server

```bash
cargo run -p standalone
```

The server will run migrations automatically on startup.

### 5. OpenAPI

- JSON schema: `http://localhost:8080/api-docs/openapi.json`
- Swagger UI: `http://localhost:8080/swagger-ui`

## API Snapshot

Base URL: `http://localhost:8080/api/v1`

Authentication:

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

## Example Flow (curl)

```bash
# 1) Register
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "email": "alice@example.com",
    "password": "secret123",
    "organization_name": "Acme",
    "organization_key": "acme"
  }'

# 2) Login and extract token (manual copy from response)
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123"}'

# 3) Create repository
curl -X POST http://localhost:8080/api/v1/repos \
  -H "Authorization: Bearer <TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{
    "organization_id":"<ORG_ID>",
    "key":"platform",
    "name":"platform",
    "visibility":"private",
    "default_branch":"main"
  }'
```

## Roadmap

### Phase 0: Foundation (done / mostly done)

- [x] Remove DI container and use explicit `AppState` assembly
- [x] Auth + JWT + organization ownership model
- [x] Invitation tokens + revoke + expiry cleanup
- [x] Repo/branch/commit base domain tables and APIs

### Phase 1: Git Data Plane

- [ ] Real git object storage and refs management
- [ ] `git push` / `git fetch` smart HTTP endpoints
- [ ] Branch updates tied to actual refs
- [ ] Large repo performance baseline

### Phase 2: Access Control

- [ ] Repository-level roles (Owner/Maintainer/Developer/Reporter)
- [ ] Fine-grained protected branch rules
- [ ] Deploy/user/group tokens
- [ ] Audit logs for permission-sensitive operations

### Phase 3: Collaboration Core

- [ ] Issues
- [ ] Merge requests
- [ ] Code review comments
- [ ] Labels, milestones, assignees

### Phase 4: CI/CD Minimal Loop

- [ ] Pipeline definitions (`.gity-ci.yml`)
- [ ] Runner registration and job dispatch
- [ ] Artifact upload/download
- [ ] Basic cache support

### Phase 5: Operations & Production Readiness

- [ ] Observability (metrics, structured audit events, traces)
- [ ] Backup/restore tooling
- [ ] Horizontal scaling strategy
- [ ] Upgrade/migration playbooks

### Phase 6: UX and Ecosystem

- [ ] Web UI for org/repo/project management
- [ ] Admin console
- [ ] CLI tooling
- [ ] Webhooks and integrations

## Configuration

Config sources (merged in order):

1. Rust defaults
2. `gity.toml`
3. `GITY_*` environment variables

Example env keys:

- `GITY_SERVER_PORT`
- `GITY_DATABASE_URL`
- `GITY_CACHE_CACHE_TYPE` (`REDIS` or `MEMORY`)
- `GITY_CACHE_URL`
- `GITY_AUTH_JWT_SECRET`

## Notes

- This project is actively evolving and not production-ready yet.
- API contracts and schema may change as core Git and collaboration features are added.

## Contributing

Contributions are welcome. Open an issue first for major changes so architecture and scope can be aligned early.
