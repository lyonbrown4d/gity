<div lang="zh-CN">

[简体中文](api-examples.md) · [English](../en/api-examples.md)

# API 示例与权限说明

本文汇总了常与 Web 界面配合使用的工作流请求与响应示例，包括项目成员、合并请求审批规则和 CI/CD 变量。

以下示例均基于这些条件：

- 基础 URL：`http://localhost:8080`
- API 路径前缀：`/api/v1`
- 调用方持有有效的 Bearer 令牌：`Authorization: Bearer <token>`

## 项目成员

### 列出成员

```bash
curl -X GET "http://localhost:8080/api/v1/projects/100/members" \
  -H "Authorization: Bearer <token>"
```

### 添加成员

```bash
curl -X POST "http://localhost:8080/api/v1/projects/100/members" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": 2001,
    "role": "developer"
  }'
```

### 更新成员角色

```bash
curl -X PATCH "http://localhost:8080/api/v1/projects/100/members/2001" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "role": "maintainer"
  }'
```

### 移除成员

```bash
curl -X DELETE "http://localhost:8080/api/v1/projects/100/members/2001" \
  -H "Authorization: Bearer <token>"
```

### 响应示例

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

### 删除操作的响应示例

```json
{
  "body": {
    "status": "deleted"
  }
}
```

> 权限要求：
> - 读取：仓库读取权限
> - 写入：仓库管理权限
> - 成员范围：响应中的 `project` 表示该成员由此项目直接管理。

## 合并请求审批规则

### 列出规则

```bash
curl -X GET "http://localhost:8080/api/v1/projects/100/merge-request-approval-rules" \
  -H "Authorization: Bearer <token>"
```

### 创建规则

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

省略 `target_branch` 时，默认值为 `*`。

### 更新规则

```bash
curl -X PATCH "http://localhost:8080/api/v1/projects/100/merge-request-approval-rules/3001" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "approvals_required": 3,
    "code_owner": true
  }'
```

### 删除规则

```bash
curl -X DELETE "http://localhost:8080/api/v1/projects/100/merge-request-approval-rules/3001" \
  -H "Authorization: Bearer <token>"
```

### 响应示例

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

### 删除操作的响应示例

```json
{
  "body": {
    "deleted": true
  }
}
```

> 权限要求：
> - 读取：项目的合并请求合并权限
> - 写入：合并请求审批规则管理权限
> - 目标分支可以是明确的分支名（如 `main`），也可以是通配符（`*`）。

## 项目 CI 变量

### 列出变量

```bash
curl -X GET "http://localhost:8080/api/v1/projects/100/ci/variables" \
  -H "Authorization: Bearer <token>"
```

### 新增或更新变量

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

掩码变量：

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

### 删除变量

```bash
curl -X DELETE "http://localhost:8080/api/v1/projects/100/ci/variables/BUILD_TAG" \
  -H "Authorization: Bearer <token>"
```

### 响应示例

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

### 删除操作的响应示例

```json
{
  "body": {
    "deleted": true
  }
}
```

> 权限要求：
> - 列出、新增或更新、删除 CI 变量均需要 Runner 管理权限。
> - 列表响应不会为掩码变量返回 `value`。

## Runner 令牌接口

`/runners/*` 接口供 Runner 代理使用，需提供 Runner 注册令牌。以上示例用于项目级管理。领取任务、追加日志和处理制品时，请使用 `POST /runners/jobs/{job_id}/...`，并在请求中携带 Runner 令牌。

</div>
