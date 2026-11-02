<div lang="en">

[简体中文](../zh/gitlab-like-domain-model.md) · [English](gitlab-like-domain-model.md)

# Gity Domain Model

This page describes the main domain objects, path rules, and permission relationships in the current Rust implementation. Versioned Rust migrations in `crates/gity-migrations` define the data model; see [Database migrations](database-migrations.md) for initialization and upgrade prerequisites.

## Organizations and Projects

### Organization

The current fields in the `organizations` table are:

- `id`: stable primary key.
- `name`: display name.
- `path_key`: organization path segment.
- `full_path`: unique full path used for routing and display.
- `description`, `visibility`, `created_at`, and `updated_at`.

Organizations have membership records in `organization_members` and can own projects. The current schema has no `parent_organization_id` foreign key or separate parent-child relationship field. Path code identifies existing descendants by segments of `full_path`; renaming an organization updates matching organization and project paths. A path prefix therefore must not be described as an explicit parent-child foreign-key relationship.

### Project

The current fields in the `projects` table are:

- `id` and `organization_id`: stable project identifier and owning organization.
- `name`, `path_key`, and `full_path`: display name, path segment, and unique full path.
- `visibility`, `description`, `default_branch`, `status`, `deleted_at`, `created_at`, and `updated_at`.

A project scopes Git repositories, members, branch protection, issues, merge requests, pipelines, jobs, runners, wikis, packages, releases, LFS, and audit capabilities. Domain tables reference projects through `project_id`; API routes for many resources use the project ID.

## Paths and Repository Storage

A project’s `full_path` is a mutable logical path and changes when its organization or project is renamed. Bare Git repositories use the immutable project ID for physical storage: the server’s `repository::repo` maps the project ID to `{id}.git`. Renaming a path therefore changes the URL, not the physical repository directory or Git data identity.

Full-path fields have a uniqueness constraint. Do not treat `full_path` as a stable primary key or describe the physical repository directory as derived from the organization name or project path.

## Member Roles and Authorization

User roles, from lowest to highest, are `guest`, `reporter`, `developer`, `maintainer`, and `owner`. Super administrators use a separate administrator authorization path.

A project user’s effective role is the higher of:

- The user’s project role in `project_members`.
- The user’s organization role in `organization_members` for the project’s organization.

Project operations also map to minimum roles. Reads usually do not require membership, but visibility controls unauthenticated access. Comments and some read operations require `reporter`; write operations generally require `developer`; repository administration, merging, and runner administration require `maintainer`; deleting a project requires `owner`. The server-side access-control code determines the action and threshold for each endpoint.

Project access tokens are independent of user roles and are limited to their project and declared scopes. They do not inherit the creator’s administrator or member roles. When reviewing permission changes, check the endpoint’s `ProjectAction`, scope mapping, and server-side checks; hiding a frontend button is not sufficient.

## Work Item Modeling

### Issue

The current fields in the `project_issues` table are:

- `id`, `project_id`, and `iid`: stable key, owning project, and the project-scoped number allocated by `project_iid_counters`.
- `author_user_id`, `title`, `description`, and `state`: author, title, description, and `opened`/`closed` state.
- `milestone_id`: the owning milestone; the server rejects milestones from other projects.
- `due_date` and `weight`: due date (`YYYY-MM-DD`) and weight.
- `confidential` and `issue_type`: confidentiality flag (0/1) and work item type such as `issue`.
- `closed_by_user_id`, `closed_at`, and `updated_by_user_id`: close and last-update metadata; reopening clears the close fields.
- `created_at` and `updated_at`.

### Milestone

`project_milestones` belongs to a project and holds `id`, `project_id`, `iid`, `title`, `description`, `state` (`active`/`closed`), `start_date`, `due_date`, and timestamps. Issues and merge requests reference milestones through `milestone_id`; deleting a milestone clears `milestone_id` on its issues. Only project-level milestones exist; there are no group milestones or iterations.

### Label

`project_labels` stores project label definitions (`project_id`, `title`, `color`, `description`) with a unique `title` per project. `project_label_links` is polymorphic: `label_id`, `target_type` (`issue`/`merge_request`), and `target_id` point at work items, with a uniqueness constraint preventing duplicate links. Older schemas inlined label name and color on the issue link table; migration V0011 aggregates existing rows into `project_labels` and drops the old table.

### Merge Request

The current fields in the `project_merge_requests` table are:

- `id`, `project_id`, `iid`, `author_user_id`, `title`, `description`, and `state` (`opened`/`closed`/`merged`).
- `source_branch`, `target_branch`, and `head_sha`: source branch, target branch, and the source commit captured at creation.
- `milestone_id`: the owning milestone.
- `merged_at`, `merged_by_user_id`, and `merge_commit_sha`: merge time, merging user, and the target branch commit after the merge; when the commit cannot be resolved the field stays empty with a warning instead of being fabricated.
- `closed_at`, `squash`, and `draft`: close time, squash marker, and draft marker. Draft merge requests are blocked from merging.

## Collaboration and Automation Objects

- **Issues and merge requests**: Issues and merge requests belong to a project; work items are identified by a project-scoped number and project ID. Membership, project labels, comments, reviewers, approval rules, and branch protection are managed by their respective project-domain objects; see the previous section for fields.
- **Pipelines and jobs**: Pipelines and jobs belong to a project and reference Git revisions such as commits, branches, or tags. Execution configuration uses `.gity-ci.yml` YAML with typed parsing and semantic validation; configuration uses typed YAML.
- **Runner**: Runners claim jobs subject to project scope, labels, and authorization. The server handles scheduling and lifecycle APIs; the Runner executes scripts.
- **Logs and artifacts**: Runners upload job traces and artifacts, which the server records and project pages and download endpoints expose. They are associated with jobs and projects; configuration-file paths are not their identity.
- **Releases, packages, wikis, LFS, and audits**: These are persisted and authorized as separate project-scoped capabilities. See the corresponding Rust migrations and API schemas for their exact fields.

## Database Migration Boundaries

The current Rust schema is defined by versioned Rust migrations in `crates/gity-migrations` for SQLite, PostgreSQL, and MySQL. The initial baseline requires an empty database; an incompatible existing schema or migration history causes migration to fail. See [Database migrations](database-migrations.md) for operating prerequisites.

New deployments use an empty database initialized from the current schema definitions.

## Code References

Review the core definitions in:

- Versioned table definitions: `crates/gity-migrations`.
- Organization/project application services and adapters: `crates/gity-projects/`, `crates/gity-platform/src/projects/`.
- Role resolution and access actions: `crates/gity-platform/src/access.rs`, `crates/gity-platform/src/access/`.
- Git storage name selection by project ID: `crates/gity-platform/src/repository/mod.rs`.
- YAML configuration loading and validation: `crates/gity-platform/src/ci/config.rs`, `crates/gity-platform/src/ci/service.rs`.
- Migration execution entry point: `crates/gity-core/src/database`; migration definitions: `crates/gity-migrations`.

</div>
