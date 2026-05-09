# GitLab-Like Domain Model

This document defines the target product model for Gity. It intentionally uses product-facing terms instead of leaking the older `namespace` implementation vocabulary.

## Product Vocabulary

- `Organization`: top-level collaboration boundary. It owns members, settings, visibility defaults, and projects.
- `Project`: the main GitLab-like aggregate. It owns repository data, issues, merge requests, wiki, packages, LFS, runners, pipelines, jobs, labels, milestones, and access overrides.
- `Repository`: a technical capability of a project, not a separate business aggregate.
- `Membership`: access can come from organization membership or direct project membership.
- `Path`: URL and clone path identity is derived from `organization.full_path + "/" + project.path_key`.

GitLab has a namespace concept internally, but the product-facing model is groups, projects, and experimental organizations. Gity should expose `orgs` and `projects`; `namespace` should not be part of public HTTP routes or frontend types.

References:

- GitLab groups: https://docs.gitlab.com/user/group/
- GitLab projects: https://docs.gitlab.com/user/project/
- GitLab namespaces: https://docs.gitlab.com/user/namespace/
- GitLab organizations: https://docs.gitlab.com/user/organization/

## Target Aggregate Boundaries

### Organization

Fields:

- `id`
- `name`
- `path_key`
- `full_path`
- `description`
- `visibility`
- `created_at`
- `updated_at`

Relations:

- `organization_members`
- `projects`

Future optional relation:

- `parent_organization_id` if we decide to model GitLab-like subgroups.

### Project

Fields:

- `id`
- `organization_id`
- `name`
- `path_key`
- `full_path`
- `visibility`
- `description`
- `default_branch`
- `created_at`
- `updated_at`

Relations:

- `project_members`
- `project_branch_protections`
- `project_issues`
- `project_merge_requests`
- `project_wiki_pages`
- `project_packages`
- `project_lfs_objects`
- `project_runners`
- `project_pipelines`
- `project_jobs`

Rules:

- `project.full_path` is globally unique.
- `project.path_key` is unique inside one organization.
- Repository storage and clone URL are derived from the project full path.
- Repository APIs live under `/projects/{id}/repository/*`.

### Access

Access levels should be normalized instead of free-form role strings:

- `guest`
- `reporter`
- `developer`
- `maintainer`
- `owner`

Rules:

- Organization membership grants inherited access to all child projects.
- Project membership can grant direct access when a user is not an organization member or needs a stronger role.
- Protected branches evaluate project role first, then inherited organization role.

### Collaboration

Issues and merge requests must be project-scoped work items:

- Keep global Snowflake `id`.
- Keep project-local `iid`.
- Add uniqueness on `(project_id, iid)`.
- Add labels, milestones, assignees, reviewers, and generic notes/discussions as the next model expansion.

### CI/CD

Pipelines, pipeline jobs, logs, artifacts, and runners are project-scoped:

- Runner scope starts at project-level.
- Organization-level runners can be added later as an inherited runner pool.
- Plano DSL remains the CI config language boundary.

## Migration Status

Completed:

- Public HTTP and frontend vocabulary uses `org` / `organization`, not `namespace`.
- Domain, application, persistence, and HTTP packages use `organization`.
- Schema resources use `organizations` and `organization_members`.
- Project ownership uses `project.organization_id`.

Remaining:

- Introduce `project_members` before hardening auth and protected branch checks.
- Add unique constraints for project-local `iid` resources.
- Add labels, milestones, assignees, reviewers, and generic notes/discussions.
