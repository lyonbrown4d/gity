# Gity Roadmap

Last updated: 2026-05-14

This document tracks the current Go-first implementation. The old Rust and Java rewrite notes are no longer part of the active plan.

## Product Direction

Gity is a lightweight GitLab-like Git hosting platform:

- Go backend using `arcgolabs/dix`, `httpx`, `authx`, `configx`, `dbx`, and related arcgolabs libraries
- TypeScript frontend using refine, React, Vite, and shadcn/ui components
- SQL migration scripts per database dialect
- Git data plane backed by native Git subprocesses where protocol correctness matters
- Repository collaboration model based on organization, project, project member, issue, merge request, CI, package, LFS, wiki, and audit domains

## Current Baseline

Available now:

- Split binaries: `gity-server`, `gity-migration`, `gity-worker`, `gity-standalone`, `gity-runner`
- Standalone mode runs migration, server, and worker in one process through `dix` subapps
- SQL migrations for sqlite, postgres, and mysql dialect folders
- User, organization, organization member, project, and project member models
- Project-level permissions with project member override and organization member fallback
- Git Smart HTTP fetch/push endpoints
- Repository browsing: branches, commits, tree, blobs, README, language stats, code search
- Branch protection with push/merge access levels, force-push/delete controls, MR requirement, and pipeline requirement
- Issues with comments, labels, assignees, and attachments
- Merge requests with diff, comments, reviewers, assignees, approvals, merge checks, and merge execution
- MR approval rules with branch scope, eligible approvers, and CODEOWNERS-backed ownership checks
- Wiki pages
- Git LFS object and lock management
- Package registry baseline
- CI pipeline/job lifecycle using Plano DSL
- Runner registration, heartbeat, job claim, trace, artifact upload, source archive download, tag matching, masked variables, and project-scoped variables
- Audit logging through asynchronous events for sensitive project operations
- Frontend workflows for repository overview, code, branches, issues, merge requests, wiki, packages, LFS, pipelines, jobs, runners, audit, settings, members, approval rules, and CI variables
- GitHub Actions and GoReleaser beta release pipeline with GitHub Release artifacts and Docker images

## Beta Boundary

The beta target is self-hosted single-node usage with sqlite or a configured SQL database.

Expected beta quality:

- Core Git fetch/push works through HTTP
- Project permissions are enforced on repository, issue, MR, CI, package, wiki, audit, and settings APIs
- SQL migrations are the only schema management path
- Server and worker do not auto-manage schema; `gity-migration` or `gity-standalone` owns migration execution
- CI runner is usable for trusted projects and trusted runner hosts
- Frontend supports the normal repository collaboration workflow without requiring direct API calls for common tasks

Explicit beta limitations:

- Runner execution is process-based, not container/VM sandboxed
- Runner secret masking is best-effort string redaction, not a formal secret-leak prevention boundary
- CODEOWNERS support covers practical path matching, section ignoring, and last-match ownership, but not every GitLab edge case
- Approval rules are project-level and branch-scoped, not yet group-inherited or per-target policy composed
- Package registry is baseline storage/metadata, not yet every ecosystem protocol
- Code search has an index baseline, but ranking and incremental invalidation are still early
- HA deployment, backup/restore tooling, and disaster recovery are documented goals, not beta requirements

## Milestones

### Milestone 1: Beta Stabilization

Status: in progress

Scope:

- Full `go test ./...` and frontend build before beta tag
- Smoke data and local beta smoke script/documentation
- ROADMAP, README, and deployment docs aligned with current binaries and SQL migrations
- Confirm sqlite local setup, split-process setup, and standalone setup
- Fix high-signal lint issues without `nolint` escape hatches

Exit criteria:

- Local `gity-standalone` can migrate and boot cleanly
- A user can create org/project, push repository data, create issue, open MR, approve, merge, run pipeline, and inspect audit events
- GitHub release workflow produces expected artifacts from a beta tag

### Milestone 2: Permission and Collaboration Hardening

Status: in progress

Scope:

- Project member management UI and API workflows
- MR approval rule UI and branch-scoped approval operations
- CODEOWNERS behavior closer to GitLab semantics
- Protected branch merge/push enforcement through HTTP Git transport and API merge path
- Audit events for permission-sensitive project operations

Exit criteria:

- Project-level role can be managed independently of organization role
- MR merge status clearly reports branch protection, pipeline, approval, and CODEOWNERS blockers
- Frontend exposes common permission-sensitive workflows without manual API calls

### Milestone 3: CI Runner Security Baseline

Status: in progress

Scope:

- Runner tag matching and project scope matching
- Masked and protected project CI variables
- Runner shell allowlist
- Workspace cleanup after job execution
- Lease expiry checks before trace, artifact, source archive, completion, and failure operations
- Runner documentation that clearly states trusted-runner assumptions

Exit criteria:

- A runner cannot claim jobs outside its project scope
- A runner only claims jobs whose tags are satisfied
- Masked values are not returned from variable list APIs and are redacted from runner traces
- Runner agent refuses non-allowlisted shells by default
- Job workspace cleanup is enabled by default for configured runners

### Milestone 4: Product Completeness

Status: planned

Scope:

- More complete package registry protocols
- MR discussions beyond flat comments
- Issue boards and milestones
- Release/tag management
- Deploy keys, deploy tokens, and project access tokens
- Webhook and notification foundation

Exit criteria:

- A small team can run common source hosting and collaboration workflows fully inside Gity
- Admins can integrate Gity with external automation without sharing personal tokens

### Milestone 5: Production Operations

Status: planned

Scope:

- Metrics and dashboard guide
- Backup and restore commands/guides
- Upgrade rehearsal documentation
- Retention policies for artifacts, packages, LFS, audit logs, and job traces
- Search index rebuild and repair operations

Exit criteria:

- Operators have documented recovery procedures
- Operational data growth has configurable retention
- Upgrades can be rehearsed against a copied database and repository root

## Near-Term Backlog

High priority:

- Run full backend test suite and frontend build after the current permission/MR/runner work
- Add beta smoke flow that covers push -> MR -> approval -> merge -> pipeline -> audit
- Finish runner trusted-host documentation
- Add API docs/examples for project members, approval rules, and CI variables

Medium priority:

- Improve CODEOWNERS parser coverage for escaped spaces and richer pattern edge cases
- Add frontend affordances for explaining why a merge is blocked
- Add package registry protocol-specific smoke tests
- Add search index rebuild command

Deferred:

- Container/VM runner executor isolation
- Multi-node HA topology
- Full GitLab-compatible CODEOWNERS and approval inheritance model
- Advanced CI variable scoping by environment/protected refs/tags
