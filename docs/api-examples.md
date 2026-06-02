# API Examples and Permission Notes

This document collects practical request/response examples for workflows that are
often used together with the web UI: project members, merge request approval rules,
and CI/CD variables.

All examples assume:

- Base URL: `http://localhost:8080`
- API path prefix: `/api/v1`
- Caller has a valid Bearer token: `Authorization: Bearer <token>`

## Project Members

### List members

```bash
curl -X GET "http://localhost:8080/api/v1/projects/100/members" \
  -H "Authorization: Bearer <token>"
```

### Add a member

```bash
curl -X POST "http://localhost:8080/api/v1/projects/100/members" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": 2001,
    "role": "developer"
  }'
```

### Update member role

```bash
curl -X PATCH "http://localhost:8080/api/v1/projects/100/members/2001" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "role": "maintainer"
  }'
```

### Remove member

```bash
curl -X DELETE "http://localhost:8080/api/v1/projects/100/members/2001" \
  -H "Authorization: Bearer <token>"
```

### Response shape

```json
{
  "body": [
    {
      "id": 12001,
      "project_id": 100,
      "user_id": 2001,
      "username": "alice",
      "display_name": "Alice Chen",
      "email": "alice@example.com",
      "role": "maintainer",
      "source": "project"
    }
  ]
}
```

### Delete response shape

```json
{
  "body": {
    "status": "deleted"
  }
}
```

> Permission baseline:
> - Read: repository read action
> - Write: repository admin action
> - Member scope: `project` entry in response means source explicitly managed by this project.

## Merge Request Approval Rules

### List rules

```bash
curl -X GET "http://localhost:8080/api/v1/projects/100/merge-request-approval-rules" \
  -H "Authorization: Bearer <token>"
```

### Create rule

```bash
curl -X POST "http://localhost:8080/api/v1/projects/100/merge-request-approval-rules" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Require two approvers",
    "target_branch": "main",
    "approvals_required": 2,
    "eligible_user_ids": [2001, 2002],
    "code_owner": false
  }'
```

`target_branch` defaults to `*` if omitted.

### Update rule

```bash
curl -X PATCH "http://localhost:8080/api/v1/projects/100/merge-request-approval-rules/3001" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "approvals_required": 3,
    "code_owner": true
  }'
```

### Delete rule

```bash
curl -X DELETE "http://localhost:8080/api/v1/projects/100/merge-request-approval-rules/3001" \
  -H "Authorization: Bearer <token>"
```

### Response shape

```json
{
  "body": {
    "project_id": 100,
    "rules": [
      {
        "id": 3001,
        "project_id": 100,
        "name": "Require two approvers",
        "target_branch": "main",
        "approvals_required": 2,
        "eligible_user_ids": [2001, 2002],
        "code_owner": true
      }
    ]
  }
}
```

### Delete response shape

```json
{
  "body": {
    "deleted": true
  }
}
```

> Permission baseline:
> - Read: merge request merge action on project
> - Write: merge request approval rule admin action
> - Target branch can be explicit branch name (`main`) or wildcard (`*`).

## Project CI Variables

### List variables

```bash
curl -X GET "http://localhost:8080/api/v1/projects/100/ci/variables" \
  -H "Authorization: Bearer <token>"
```

### Upsert variable

```bash
curl -X PATCH "http://localhost:8080/api/v1/projects/100/ci/variables" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "key": "BUILD_TAG",
    "value": "release-2026-05-16",
    "masked": false,
    "protected": true
  }'
```

Masked variable:

```bash
curl -X PATCH "http://localhost:8080/api/v1/projects/100/ci/variables" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "key": "TOKEN",
    "value": "replace-me-very-long-value",
    "masked": true,
    "protected": false
  }'
```

### Delete variable

```bash
curl -X DELETE "http://localhost:8080/api/v1/projects/100/ci/variables/BUILD_TAG" \
  -H "Authorization: Bearer <token>"
```

### Response shape

```json
{
  "body": [
    {
      "id": 5001,
      "project_id": 100,
      "key": "BUILD_TAG",
      "value": "release-2026-05-16",
      "masked": false,
      "protected": true
    },
    {
      "id": 5002,
      "project_id": 100,
      "key": "SECRET_KEY",
      "masked": true,
      "protected": false
    }
  ]
}
```

### Delete response shape

```json
{
  "body": {
    "deleted": true
  }
}
```

> Permission baseline:
> - CI variable list/upsert/delete requires runner admin action
> - A masked variable never returns `value` in list responses.

## Runner token endpoints

`/runners/*` endpoints are for runner agents and use runner registration tokens.
The examples above are for project-scoped management. For job claim/trace/artifact flow,
use `POST /runners/jobs/{job_id}/...` with the runner token payload.
