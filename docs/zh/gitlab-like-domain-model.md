<div lang="zh-CN">

[简体中文](gitlab-like-domain-model.md) · [English](../en/gitlab-like-domain-model.md)

# Gity 领域模型

本文描述当前 Rust 实现中的主要领域对象、路径和权限关系。数据结构以 `migrations/` 中当前版本的 Rust 迁移为准。

## 组织与项目

### Organization

组织表 `organizations` 当前字段为：

- `id`：稳定主键。
- `name`：展示名称。
- `path_key`：组织路径段。
- `full_path`：用于路由和展示的完整路径，唯一。
- `description`、`visibility`、`created_at`、`updated_at`。

组织有 `organization_members` 成员关系，可拥有项目。当前 schema 没有 `parent_organization_id` 外键或独立父子关系字段；路径代码按 `full_path` 的路径段处理已有后代，组织改名会更新匹配的组织和项目路径。因此，不能把路径前缀误写成显式父子外键关系。

### Project

项目表 `projects` 当前字段为：

- `id`、`organization_id`：稳定项目标识及所属组织。
- `name`、`path_key`、`full_path`：展示名、路径段和唯一完整路径。
- `visibility`、`description`、`default_branch`、`status`、`deleted_at`、`created_at`、`updated_at`。

项目是 Git 仓库、成员、分支保护、Issue、Merge Request、流水线、Job、Runner、Wiki、Package、Release、LFS 和审计等能力的作用域。领域表以 `project_id` 关联项目；API 中多个资源使用项目 ID 路由。

## 路径与仓库存储

项目 `full_path` 是可变的逻辑路径，组织或项目改名会调整它。Git 裸仓库物理存储使用不可变的项目 ID：服务端 `repository::repo` 将项目 ID 转为 `{id}.git`。因此路径改名影响 URL，不改变仓库物理目录和 Git 数据身份。

完整路径字段有唯一约束。不要将 `full_path` 当作稳定主键，也不要把物理仓库目录描述为由组织名或项目路径派生。

## 成员角色与授权

用户角色级别从低到高为 `guest`、`reporter`、`developer`、`maintainer`、`owner`。超级管理员走独立的管理员授权路径。

项目用户的有效级别取以下两项较高者：

- 该用户在 `project_members` 中的项目角色。
- 该用户在项目所属组织的 `organization_members` 中的组织角色。

项目操作还映射到最低角色：读取通常不要求成员身份，但可见性会控制未登录访问；评论和部分读取操作要求 reporter；写入类操作通常要求 developer；仓库管理、合并和 Runner 管理要求 maintainer；删除项目要求 owner。具体端点使用的动作及阈值由服务端访问控制代码决定。

项目访问令牌独立于用户角色，仅允许其所属项目和令牌声明的 scope。它不会继承创建者的管理员或成员角色。审查权限变更时，应核对端点的 `ProjectAction`、scope 映射和服务端检查，而不能只依据前端隐藏按钮。

## 协作与自动化对象

- **Issue 与 Merge Request**：属于项目；通过项目内序号和项目 ID 标识工作项。成员、标签、评论、审查人、审批规则和分支保护分别由各自项目域对象管理。
- **Pipeline 与 Job**：属于项目并关联提交、分支或标签等 Git 修订信息。执行配置使用 `.gity-ci.yml` YAML，经类型解析和语义校验；不支持旧 Plano DSL。
- **Runner**：领取受项目范围、标签和授权约束的 Job。服务端负责调度与生命周期接口，Runner 负责执行脚本。
- **日志与产物**：Job trace 和 artifact 通过 Runner 上传、服务端记录，再由项目页面和下载接口读取；它们关联 Job/项目，不由配置文件路径充当身份。
- **Release、Package、Wiki、LFS 与审计**：作为项目下的独立领域能力持久化和授权，具体字段以相应 Rust 迁移及 API schema 为准。

## 数据库迁移边界

当前 Rust schema 由根目录 `migrations/` 中按 SQLite、PostgreSQL、MySQL 方言维护的版本化 SQL 构成。启动迁移会校验迁移名称和校验值；空数据库可初始化，已管理数据库可按记录升级。存在未受管 schema、无效迁移历史或不匹配校验值时会拒绝继续。

新部署使用空数据库，由当前 schema 定义初始化。

## 代码依据

核心定义可从以下位置复核：

- 版本化表结构：`migrations/{sqlite,postgres,mysql}/`。
- 组织与项目路径更新：`apps/gity-server/src/projects/`。
- 角色合并和访问动作：`apps/gity-server/src/access.rs`、`apps/gity-server/src/access/`。
- 按项目 ID 选择 Git 存储名：`apps/gity-server/src/repository/mod.rs`。
- YAML 配置读取与校验：`apps/gity-server/src/ci/config.rs`、`apps/gity-server/src/ci/service.rs`。
- 迁移历史及校验：`crates/gity-core/src/database/migration.rs`。

</div>
