<div lang="en">

[简体中文](../zh/gitlab-like-domain-model.md) · [English](gitlab-like-domain-model.md)

# Gity Domain Model

This page describes the main domain objects, path rules, and permission relationships in the current Rust implementation. The current Rust migrations in `migrations/` are the source of truth for the data model.

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

## Collaboration and Automation Objects

- **Issues and merge requests**: Issues and merge requests belong to a project; work items are identified by a project-scoped number and project ID. Membership, labels, comments, reviewers, approval rules, and branch protection are managed by their respective project-domain objects.
- **Pipelines and jobs**: Pipelines and jobs belong to a project and reference Git revisions such as commits, branches, or tags. Execution configuration uses `.gity-ci.yml` YAML with typed parsing and semantic validation; configuration uses typed YAML.
- **Runner**: Runners claim jobs subject to project scope, labels, and authorization. The server handles scheduling and lifecycle APIs; the Runner executes scripts.
- **Logs and artifacts**: Runners upload job traces and artifacts, which the server records and project pages and download endpoints expose. They are associated with jobs and projects; configuration-file paths are not their identity.
- **Releases, packages, wikis, LFS, and audits**: These are persisted and authorized as separate project-scoped capabilities. See the corresponding Rust migrations and API schemas for their exact fields.

## Database Migration Boundaries

The current Rust schema is defined by versioned SQL migrations for SQLite, PostgreSQL, and MySQL under the repository-root `migrations/` directory. Startup migrations validate migration names and checksums. Empty databases can be initialized, and managed databases can be upgraded according to their recorded history. Migration startup refuses to continue if it finds an unmanaged schema, invalid migration history, or a checksum mismatch.

New deployments use an empty database initialized from the current schema definitions.

## Code References

Review the core definitions in:

- Versioned table definitions: `migrations/{sqlite,postgres,mysql}/`.
- Organization and project path updates: `apps/gity-server/src/projects/`.
- Role resolution and access actions: `apps/gity-server/src/access.rs`, `apps/gity-server/src/access/`.
- Git storage name selection by project ID: `apps/gity-server/src/repository/mod.rs`.
- YAML configuration loading and validation: `apps/gity-server/src/ci/config.rs`, `apps/gity-server/src/ci/service.rs`.
- Migration history and checksum validation: `crates/gity-core/src/database/migration.rs`.

</div>
