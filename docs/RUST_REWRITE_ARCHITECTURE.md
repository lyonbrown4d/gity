# Rust Rewrite Architecture

This document defines the target architecture for the Rust rewrite.

## Goals

- Replace the old `entity` + `repository` + `sea-orm` stack with a Toasty-first stack.
- Align the domain model with a GitLab-like product shape.
- Treat `project` as the core product aggregate.
- Treat Git repository storage as infrastructure owned by the project, not as the product aggregate itself.
- Avoid forward-compatibility code and one-off migration shims.

## Non-goals

- Preserve the current database schema.
- Keep the old repository-first service boundaries.
- Dual-run SeaORM and Toasty in the main path for a long period.

## Workspace Direction

Target crate layout:

- `apps/standalone`
  HTTP API, auth middleware, serialization, bootstrapping.
- `crates/models`
  All Toasty models. This becomes the single source of truth for application data.
- `crates/application`
  Use cases, command handlers, query handlers, permission checks, transaction orchestration.
- `crates/platform`
  Toasty database runtime, configuration, migrations/schema management entry points.
- `crates/git`
  Bare repository storage, refs, object access, smart HTTP transport.

Legacy crates to retire:

- `crates/entity`
- `crates/repository`
- `crates/migration`

## Product Model

The new core hierarchy is:

- `user`
- `namespace`
- `namespace_member`
- `project`
- `project_issue`
- `project_issue_comment`

GitLab-like interpretation:

- `namespace` is the internal top-level concept.
- A personal space is a `user` namespace.
- An organization or team space is a `group` namespace.
- Subgroups are modeled as nested namespaces via `parent_namespace_id`.
- `project` lives under a namespace and owns collaboration features.

UI wording can still use `organization` or `team` when helpful, but the internal model stays namespace-first.

Future additions build on top of the same project aggregate:

- merge requests
- labels
- attachments
- package registry
- LFS
- tokens and deploy credentials

## API Cut-over

The rewrite does not use a staging `/api/v2` namespace.

The new primary write surface is:

- `/api/v1/namespaces`
- `/api/v1/projects`

Legacy endpoints such as `/api/v1/orgs` and `/api/v1/repos` may remain mounted temporarily while their behavior is migrated, but all new product modeling and new write paths should land on the namespace/project API.

## Data Model Principles

- Internal primary keys use `i64`.
- Public routing identifiers are modeled explicitly and are not coupled to internal primary keys.
- `namespace.full_path` and `project.full_path` are first-class routing fields.
- `path_key` stores the GitLab-like path segment for namespaces and projects.
- `iid` values are scoped to a project for issues and later for merge requests.
- `project` owns user-facing collaboration features.
- Git branches and commits are read from the Git object store and are not treated as primary relational truth.
- Relational tables only persist collaboration metadata and durable product state.

## Delivery Phases

### Phase 1

- Add Toasty-based model crate.
- Add platform crate for Toasty runtime and schema tooling entry points.
- Add application crate for new use-case boundaries.
- Keep the old code compiling while the new path takes shape.

### Phase 2

- Rebuild auth, namespace, and project creation on top of the new models.
- Move the new namespace/project workflow onto `/api/v1`.
- Stop adding features to the SeaORM path.

### Phase 3

- Rebuild issues and comments on the new project-centric schema.
- Shift Git repository initialization to project creation.
- Remove repository-first persistence concepts from the HTTP surface.

### Phase 4

- Remove `entity`, `repository`, and `migration`.
- Move package registry, attachments, and LFS onto the new project-centric data model.
- Finish the GitLab-like platform surface.

## Toasty Policy

- Toasty is the only ORM and schema-management stack for the rewrite.
- The project should pin a specific Toasty version during the rewrite window.
- Breaking API changes from upstream should be absorbed intentionally, not implicitly.
