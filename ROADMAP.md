# Gity Roadmap

Last updated: 2026-06-27

This document tracks the active Go-first product and engineering plan. The old Rust, Java, SeaORM, and JWT rewrite notes are no longer part of the active roadmap.

## Product Direction

Gity is a lightweight GitLab-like Git hosting platform for small teams and self-hosted deployments.

- Go backend using `arcgolabs/dix`, `httpx`, `authx`, `configx`, `dbx`, and related arcgolabs libraries.
- TypeScript frontend using refine, React, Vite, and shadcn/ui components.
- SQL migration scripts per database dialect.
- Git data plane backed by native Git subprocesses where protocol correctness matters.
- Repository collaboration model based on organization, project, project member, issue, merge request, CI, package, LFS, wiki, release, credential, runner, and audit domains.

The product target is not API surface completeness alone. The beta target is that a human user can complete the common Git platform loops without direct API calls.

## Current Baseline

Available now:

- Split binaries: `gity-server`, `gity-migration`, `gity-search-index`, `gity-worker`, `gity-standalone`, `gity-runner`.
- Standalone mode runs migration, server, and worker in one process through `dix` subapps.
- SQL migrations for sqlite, postgres, and mysql dialect folders.
- User, organization, organization member, project, and project member models.
- Project-level permissions with project member override and organization member fallback.
- Git Smart HTTP fetch/push endpoints.
- Repository browsing: branches, commits, tree, blobs, README, language stats, and code search.
- Branch protection with push/merge access levels, force-push/delete controls, MR requirement, and pipeline requirement.
- Issues with comments, labels, assignees, and attachments.
- Merge requests with diff, comments, reviewers, assignees, approvals, merge checks, and merge execution.
- MR approval rules with branch scope, eligible approvers, and CODEOWNERS-backed ownership checks.
- Wiki pages.
- Git LFS object and lock management.
- Package registry baseline with common protocol endpoints in progress.
- Release and Git tag management with release asset links.
- CI pipeline/job lifecycle using Plano DSL.
- Runner registration, heartbeat, job claim, trace, artifact upload, source archive download, tag matching, masked variables, and project-scoped variables.
- Project access tokens, deploy tokens, and deploy key storage.
- Audit logging through asynchronous events and dbx row audit support for sensitive project operations.
- Frontend workflows for repository overview, code, branches, issues, merge requests, wiki, packages, LFS, pipelines, jobs, runners, audit, settings, members, approval rules, CI variables, releases, and credentials.
- GitHub Actions and GoReleaser beta release pipeline with GitHub Release artifacts and Docker images.
- Integration, blackbox, Git client, API, and k6 pressure-test scripts.

## Product Loops

These loops define product readiness. A feature is not complete until the loop works from the UI or normal developer tooling.

1. Repository bootstrap: create or import project -> create credential -> clone -> push -> browse README/code/commits.
2. Collaboration: create issue -> assign/label -> create branch -> open MR -> review -> approve -> merge -> audit.
3. CI/CD: register runner -> lint pipeline -> execute job -> inspect live log -> download artifact -> retry/cancel when needed.
4. Package/release: publish package or release -> view versions/assets -> copy install command -> enforce retention/delete rules.
5. Operations: admin reviews users/projects/runners/storage/audit -> applies settings -> verifies backup/restore and upgrade path.

## Beta Boundary

The beta target is self-hosted single-node usage with sqlite or a configured SQL database.

Expected beta quality:

- Core Git fetch/push works through HTTP.
- Project permissions are enforced on repository, issue, MR, CI, package, wiki, release, audit, credential, and settings APIs.
- SQL migrations are the only schema management path.
- Server and worker do not auto-manage schema; `gity-migration` or `gity-standalone` owns migration execution.
- CI runner is usable for trusted projects and trusted runner hosts.
- Frontend supports the normal repository collaboration workflow without requiring direct API calls for common tasks.
- Empty states teach the next action instead of only saying no data exists.

Explicit beta limitations:

- Runner execution is trusted-host by default; container runtimes are supported as a baseline but not a complete hostile-code sandbox.
- Firecracker support is experimental and not a complete VM runner yet.
- Runner secret masking is best-effort string redaction, not a formal secret-leak prevention boundary.
- CODEOWNERS support covers practical path matching, section ignoring, and last-match ownership, but not every GitLab edge case.
- Approval rules are project-level and branch-scoped, not yet group-inherited or per-target policy composed.
- Package registry protocol coverage is baseline and not fully ecosystem-compatible yet.
- Code search has an index baseline, but ranking and incremental invalidation are still early.
- SSH Git transport is not yet a beta requirement; stored deploy keys are preparation for SSH/deploy workflows.
- HA deployment, backup/restore tooling, and disaster recovery are documented goals, not beta requirements.

## Milestones

### Milestone 1: Beta Stabilization

Status: in progress

Scope:

- Full `go test ./...`, frontend typecheck/build, release check script, and blackbox smoke before beta tag.
- Smoke data, local beta smoke documentation, and beta release checklist.
- ROADMAP, README, deployment docs, and API examples aligned with current binaries and SQL migrations.
- Confirm sqlite local setup, split-process setup, and standalone setup.
- Fix high-signal lint issues without `nolint` escape hatches.
- Keep pressure-test scripts cross-platform and runnable through Node/PowerShell.

Exit criteria:

- Local `gity-standalone` can migrate and boot cleanly.
- A user can create org/project, push repository data, create issue, open MR, approve, merge, run pipeline, and inspect audit events.
- GitHub release workflow produces expected artifacts from a beta tag.

### Milestone 2: Product Entry and Navigation

Status: in progress

Scope:

- Project overview that summarizes README, clone commands, recent commits, open issues, open MRs, latest pipeline, runners, packages, and audit signals.
- Global quick jump for projects, files, issues, merge requests, and common settings.
- First-run and empty-state guidance for creating orgs, projects, credentials, runners, and packages.
- Credential UX that explains project access tokens, deploy tokens, deploy keys, and clone command usage.

Exit criteria:

- A new user can get from login to first push without reading backend docs.
- Common project destinations are reachable through one global shortcut.
- Every empty tab has a next action and a permission-aware explanation.

### Milestone 3: Permission and Collaboration Hardening

Status: in progress

Scope:

- Project member management UI and API workflows.
- MR approval rule UI and branch-scoped approval operations.
- CODEOWNERS behavior closer to GitLab semantics.
- Protected branch merge/push enforcement through HTTP Git transport and API merge path.
- Audit events for permission-sensitive project operations.
- Issue filters, saved views, labels, assignees, milestones, and board-ready state transitions.
- MR discussion model beyond flat comments.

Exit criteria:

- Project-level role can be managed independently of organization role.
- MR merge status clearly reports branch protection, pipeline, approval, and CODEOWNERS blockers.
- Frontend exposes common permission-sensitive workflows without manual API calls.
- Users can triage issue and MR work through project-scoped views.

### Milestone 4: CI Runner Security and Usability

Status: in progress

Scope:

- Runner tag matching and project scope matching.
- Masked and protected project CI variables.
- Runner shell allowlist.
- Workspace cleanup after job execution.
- Lease expiry checks before trace, artifact, source archive, completion, and failure operations.
- Pipeline graph, live job trace, retry/cancel controls, artifact browser, and CI lint editor.
- Runner documentation that clearly states trusted-runner assumptions and runtime isolation choices.

Exit criteria:

- A runner cannot claim jobs outside its project scope.
- A runner only claims jobs whose tags are satisfied.
- Masked values are not returned from variable list APIs and are redacted from runner traces.
- Runner agent refuses non-allowlisted shells by default.
- Job workspace cleanup is enabled by default for configured runners.
- Users can diagnose failed pipelines from the UI.

### Milestone 5: Package, Release, and Automation Completeness

Status: planned

Scope:

- More complete package registry protocols and protocol-specific install/publish snippets.
- Release notes, assets, and tag-oriented project history.
- Deploy keys, deploy tokens, and project access token workflows.
- Webhook and notification foundation.
- Todo/inbox for assigned issues, review requests, mentions, failed pipelines, and approvals needed.

Exit criteria:

- A small team can run common source hosting, CI, package, and release workflows fully inside Gity.
- Admins can integrate Gity with external automation without sharing personal tokens.

### Milestone 6: Production Operations

Status: planned

Scope:

- Metrics and dashboard guide.
- Backup and restore commands/guides.
- Upgrade rehearsal documentation.
- Retention policies for artifacts, packages, LFS, audit logs, and job traces.
- Search index rebuild and repair operations.
- Storage usage, quota, rate limit, and abuse-prevention controls.

Exit criteria:

- Operators have documented recovery procedures.
- Operational data growth has configurable retention.
- Upgrades can be rehearsed against a copied database and repository root.

## Near-Term Implementation Plan

High priority:

- Project overview/home experience with workflow status and clone guidance.
- Global quick jump and search entry point.
- Credential guidance for project access tokens, deploy tokens, deploy keys, and HTTP clone commands.
- Issue and MR triage affordances: filters, labels, assignees, merge blockers, and next-action summaries.
- CI usability: job logs, artifacts, retry/cancel actions, runner health, variable masking explanation.
- Release-check, integration, blackbox, and k6 pressure-test coverage for the above user loops.

Medium priority:

- Issue milestones and board-ready workflow states.
- MR discussion threading and resolved/unresolved state.
- CODEOWNERS parser coverage for escaped spaces and richer pattern edge cases.
- Package registry protocol-specific smoke tests and install snippets.
- Search index rebuild command hardening and incremental invalidation strategy.

Deferred:

- Full SSH Git transport.
- Hostile-code runner sandbox guarantee.
- Multi-node HA topology.
- Full GitLab-compatible CODEOWNERS and approval inheritance model.
- Advanced CI variable scoping by environment/protected refs/tags.
