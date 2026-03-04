# Gity Roadmap

This document defines a practical delivery plan for Gity.
It is intended to be updated as milestones move from planned -> in progress -> done.

## Planning Principles

- Deliver thin vertical slices instead of broad unfinished layers
- Keep behavior explicit and testable
- Treat performance and operability as first-class requirements
- Avoid irreversible design commitments too early

## Current Baseline

Available now:

- JWT-based auth and basic organization model
- Invitation lifecycle (create/accept/revoke/expire)
- Repository, branch, and commit metadata APIs
- Postgres schema via SeaORM migrations
- Optional Redis cache client initialization

Known gaps:

- No full Git object/refs data plane yet
- No repository-level role matrix
- No merge request/issue workflow
- No CI pipeline/job lifecycle
- No production-grade observability pack

## Milestone Plan

### Milestone 1: Git Data Plane (Foundation)

Objective:

- Make repository state backed by real Git data operations.

Scope:

- Repository storage layout conventions
- Smart HTTP endpoints for fetch/push path
- Ref updates tied to branch APIs
- Basic authorization checks during Git operations

Exit Criteria:

- Clone/fetch from server-backed repositories works
- Push updates branch refs under permission checks
- Branch metadata and actual refs remain consistent
- Smoke tests cover core fetch/push flows

Risks:

- Git protocol complexity and edge cases
- Data consistency between metadata DB and refs

### Milestone 2: Access Control Hardening

Objective:

- Move from organization-only checks to repository-level permissions.

Scope:

- Role model: Owner/Maintainer/Developer/Reporter
- Protected branch rules by role
- Token strategy for automation (project/deploy/user token)
- Audit trail for permission-sensitive actions

Exit Criteria:

- Repository APIs enforce role-based permissions
- Protected branch behavior is deterministic and tested
- Audit events available for key write operations

Risks:

- Permission matrix complexity
- Backward compatibility for existing APIs

### Milestone 3: Collaboration Core

Objective:

- Introduce minimal code collaboration flow.

Scope:

- Issues: create/list/update/close
- Merge requests: open, update, merge checks
- Discussion threads on merge requests
- Labels and assignees (minimal set)

Exit Criteria:

- Team can track work in-repo with issues
- Merge request lifecycle works for normal cases
- Merge action is permission-checked and auditable

Risks:

- Data model expansion impacting API consistency
- UX/API contract changes if done too broadly

### Milestone 4: CI/CD Minimal Loop

Objective:

- Execute basic CI jobs tied to repository events.

Scope:

- Pipeline definition file (`.gity-ci.yml`) parser
- Runner registration and polling/dispatch
- Job logs and status model
- Artifact upload/download (basic retention)

Exit Criteria:

- A commit can trigger a pipeline with at least one job
- Runner can pick and execute jobs reliably
- Users can view job status and logs via API

Risks:

- Runner security boundaries
- Job isolation and resource control

### Milestone 5: Production Readiness

Objective:

- Raise operational confidence for self-hosted deployments.

Scope:

- Metrics, structured logs, trace correlation IDs
- Backup and restore guides/tools
- Failure mode testing and recovery playbooks
- Upgrade and migration compatibility checks

Exit Criteria:

- Core SLO signals observable (latency/error/saturation)
- Backup+restore validated on non-trivial dataset
- Upgrade path documented and exercised

Risks:

- Hidden operational coupling across components
- Incomplete disaster recovery drills

## Execution Cadence

- Keep milestones small enough for 3-6 week delivery windows
- Review scope weekly and cut non-critical items early
- Ship behind feature flags when uncertainty is high

## Suggested Next 30/60/90 Day Focus

30 days:

- Close Milestone 1 minimum viable scope
- Add integration tests for fetch/push happy path

60 days:

- Deliver Milestone 2 role and protected-branch matrix
- Add API compatibility notes and migration guidance

90 days:

- Start Milestone 3 issues + minimal merge request flow
- Define CI runner security baseline before Milestone 4

## Tracking Template

Use this for each milestone item:

- Status: planned / in progress / blocked / done
- Owner:
- Target Date:
- Dependencies:
- Verification:
- Notes:
