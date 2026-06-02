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

This is a manual flow and should be executed on a fresh `./data/gity.db` instance.

1. Open `http://localhost:8080/api/health` and expect an OK response.
2. Open `http://localhost:8080/docs` and verify OpenAPI renders.
3. Create a user and token through the user token endpoints.
4. Create an organization and project.
5. Configure branch protection for `main` on `PATCH /projects/{project_id}/repository/branch-protections/main`:
   - `rule_type=exact`
   - `push_access_level=maintainer`
   - `merge_access_level=maintainer`
   - `require_merge_request=true`
   - `require_pipeline_success=false`
   - `allow_force_push=false`
   - `allow_delete=false`
6. Confirm protected branch writes are blocked when rules require MR and only non-protected branches accept direct push.
7. Push a commit to a feature branch, open an MR into the protected target branch, approve with a non-author user, and verify merge checks require the approval.
8. Ensure MR merge is blocked if:
   - approvals are insufficient,
   - protected branch checks fail,
   - or required pipeline status is not successful.
9. Register a project runner and execute one job:
   - runner claims queued job
   - trace append succeeds
   - source archive download works
   - artifact upload succeeds
   - job completes before lease timeout
10. Validate pipeline/job observability:
   - a pipeline row is visible for the MR
   - at least one job transitions `pending` -> `running` -> `success` (or explicit failure state)
   - trace output is retrievable
11. Validate audit trail integrity:
   - project creation, branch protection changes, MR approval, merge, and runner job completion events are present
12. Verify storage persistence:
   - restart `cmd/server` with same `.env`
   - confirm data persists under `./data/gity.db`
13. Validate extra product planes:
   - package registry create/download path works for one package type
   - wiki page create/edit/read works
   - issue create/comment/edit + assignee + labels works

Any failure in the above flow should block the beta release.

The local data directory is disposable. Remove `./data` only when intentionally resetting the smoke database and repositories.
