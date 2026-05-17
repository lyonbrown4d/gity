# Beta Smoke Runbook

This runbook validates the local sqlite path before cutting a beta tag.

Use this together with [beta-release-checklist.md](beta-release-checklist.md). The checklist covers automated checks and packaging; this runbook covers product smoke behavior.

## Local Runtime

Use the checked-in example as the configx dotenv source:

```bash
cp .env.example .env
go run ./cmd/migration
go run ./cmd/server
```

For a single-process local run, use standalone instead. It runs migration first and then starts server and worker subapps:

```bash
go run ./cmd/standalone
```

Split `server` and `worker` deployments intentionally do not manage schema. Run `cmd/migration` explicitly before either process.

## Smoke Checklist

1. Open `http://localhost:8080/api/health` and expect an OK response.
2. Open `http://localhost:8080/docs` and verify OpenAPI renders.
3. Create a user and token through `/api/v1/users` and `/api/v1/users/{id}/tokens`.
4. Create an organization, project, and a protected `main` branch.
5. Confirm direct protected branch writes are blocked when `require_merge_request=true`.
6. Create an MR into the protected target branch, approve it with a non-author user, and verify merge checks require the approval.
7. Register a project runner, claim a script job, append trace, download source archive, upload artifacts, and complete the job before the lease expires.
8. Restart `cmd/server` with the same `.env` and confirm sqlite data persists under `./data/gity.db`.

The local data directory is disposable. Remove `./data` only when intentionally resetting the smoke database and repositories.
