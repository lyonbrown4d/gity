<div lang="zh-CN">

[简体中文](overview.md) · [English](../../en/architecture/overview.md)

# 整体架构

```mermaid
flowchart TB
  Web["浏览器 / HTTP Git"] -->|HTTP :8080| Gateway
  Package["语言包 / Generic 客户端"] -->|HTTP :8080| Gateway
  SSH["SSH Git"] -->|SSH :2222| Gateway
  OCI["Docker / OCI"] -->|:5000 → :8081| Registry
  Gateway["Gateway"] -->|HTTP / HTTP Git| API["API :8080"]
  Gateway -->|SSH| Git["Git server :2222"]
  Gateway -->|Packages :8082| Registry["Registry"]
```



Gity 采用共享业务库和多个可部署入口。Standalone 在一个平台进程内组合主要模块；分布式布局把入口、平台、Git 存储和制品协议服务放到不同进程。两者复用业务实现。

## 代码分层

```text
apps/                   可执行入口与进程配置
  ↓
crates/gity-platform    平台路由、HTTP 适配与应用装配
  ↓
gity-projects / gity-collaboration / gity-ci 等应用服务与领域规则
  ↓
SeaORM 类型化仓储、Git 本地或 RPC 适配、对象存储适配
```

`apps/` 不供其他应用导入业务实现。`crates/` 保存可复用组件和数据库迁移库，`xtask/` 保存构建和验证工具。数据库迁移由 [数据库迁移说明](../database-migrations.md) 介绍。

当前按身份权限、项目、协作、CI、Git、制品等上下文组织实现。项目与仓库访问由 `gity-projects` 提供应用服务；Issue、MR、里程碑、发布等协作流程由 `gity-collaboration` 提供；流水线和任务流程由 `gity-ci` 提供。`gity-platform` 保留 HTTP 路由适配、请求身份与权限检查以及运行时组合。上下文边界并不表示每个上下文拥有独立进程或数据库。持久化边界和事务说明见[业务持久化与依赖装配](typed-persistence.md)。

## 运行依赖

数据库保存业务元数据，不保存 Git 对象。Git 对象保存在裸仓库中；软件包、附件和 CI 产物通过存储适配访问。分布式示例使用 PostgreSQL 和 SeaweedFS；各平台进程仍共享该 PostgreSQL，尚未实现每个上下文独立数据库。

Web 是编译后的静态资源，由平台 HTTP 服务提供，不是独立的 Node 服务。Monaco、Milkdown 作为本地第三方资产随 Web 分发。

## 公共基础能力

| 能力 | 当前实现 |
| --- | --- |
| 装配 | FluxDI 注册、解析和 module 生命周期 |
| HTTP / 并发 | Axum、Tokio；传输路径保留流式读写 |
| 持久化 | SeaORM Entity、ActiveModel、类型化仓储与事务；跨上下文审计和 IID 计数由 `gity-persistence` 提供 |
| 实体标识 | Snowflake（ferroid，41/10/12 位）；装配阶段按唯一 node ID 安装生成器，插入时由 `before_save` 赋值；对外 JSON 主键用字符串 |
| 操作标识 | UUID v7；秘密凭据使用密码学随机数 |
| 配置 | 代码默认值 < 文件 < 环境变量 < 显式 CLI |
| 可观测性 | Tracing、Prometheus、可选 OTLP；各进程分别初始化 |
| API 文档 | Utoipa 宏生成 OpenAPI，平台提供 `/docs`、`/openapi.json` |
| 多语言 | 前后端复用 `gity-i18n`，服务端语言按请求选择 |

## 部署与保证范围

Standalone 默认包含全部业务组件，但 Runner 和 Angos 是被监督的独立子进程；Compose 还运行独立 DinD。它不等于整个安装只有一个 OS 进程。

分布式组件可独立打包和启动，但存在明确依赖，不能任意移除。当前 Compose 不是多主机、高可用或独立数据库的完整部署方案。组件职责见[依赖表](components.md)，启动与关闭见 [Standalone](standalone.md) 和[分布式模式](distributed.md)。

源码依据：[平台装配](https://github.com/lyonbrown4d/gity/blob/main/crates/gity-platform/src/composition.rs)、[应用角色](https://github.com/lyonbrown4d/gity/blob/main/crates/gity-platform/src/application.rs)、[Workspace](https://github.com/lyonbrown4d/gity/blob/main/Cargo.toml)。

</div>
