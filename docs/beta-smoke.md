# Beta Smoke Runbook

This runbook describes product flows to exercise on the Rust migration branch before a beta tag. It has not been executed as part of this documentation update.

Use this together with [beta-release-checklist.md](beta-release-checklist.md). The checklist covers automated checks and packaging; this runbook covers product smoke behavior.

## Local Runtime

Use the Rust configuration example and run the current server commands:

```bash
cp .env.example .env
cargo run -p gity-server -- migrate
cargo run -p gity-server -- serve
```

For a single-process local run, use standalone instead. It runs migration first and then starts the HTTP service and scheduler:

```bash
cargo run -p gity-server -- standalone
```

`serve` and `worker` do not manage schema. Run `migrate` before either process. For the packaged Docker image, `scripts/blackbox-e2e.ps1` checks service health, OpenAPI, and the Trunk Web root.

## Smoke Checklist

This is a manual flow and should be executed on a fresh Rust database and repository store.

1. Open `http://localhost:8080/api/health` and expect an OK response.
2. Open `http://localhost:8080/docs` and verify OpenAPI renders.
3. Create a user and token through the user token endpoints.
4. Create an organization and project.
5. As a project maintainer, configure branch protection for `main` with `POST /api/v1/projects/{project_id}/repository/branches/{branch}/protect` and a JSON body:
   - `push_access_level: "maintainer"`
   - `merge_access_level: "maintainer"`
   - `require_merge_request: true`
   - `require_pipeline_success: false`
   - `allow_force_push: false`
   - `allow_delete: false`
   The `rule_type` is inferred as `exact` for `main`; it is inferred as `pattern` for valid glob branches.
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
   - a job transitions `pending` -> `running` -> `succeeded` (or `failed`/`canceled`)
   - trace output is retrievable
11. Validate audit trail integrity:
   - project creation, branch protection changes, MR approval, merge, and runner job completion events are present
12. Verify storage persistence:
   - restart the Rust server with the same configuration
   - confirm database, repository, and object data persist
13. Validate extra product planes:
   - package registry create/download path works for one package type
   - wiki page create/edit/read works
   - issue create/comment/edit + assignee + labels works

Any failure in the above flow should block the beta release.

The local integration volumes are disposable. Remove them only when intentionally resetting the smoke database and repositories.
