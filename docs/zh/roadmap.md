<div lang="zh-CN">

[简体中文](roadmap.md) · [English](../en/roadmap.md)

# 产品范围

Gity 面向团队协作与自托管，通过 Web 和常规 Git 客户端提供仓库、Issue、MR、CI、软件包与容器仓库功能。服务端使用 Rust、Axum、Tokio、SeaORM，Web 使用 Leptos/WASM，FluxDI 负责装配。

## 当前边界

- 项目成员、组织成员和令牌 scope 共同决定有效权限，由服务端执行。
- Git 裸仓库使用不可变项目 ID 定位，公开项目路径可与物理存储名称分离。
- 数据库支持 SQLite、PostgreSQL 和 MySQL；初始化与后续版本更新通过版本化 Rust migration 管理。
- Runner 和服务端调度分离，使用可信 Runner 主机执行项目脚本。
- 软件包协议包含 Generic、Cargo、npm、PyPI、Maven、NuGet、Go；OCI 使用 Angos，mirror 上游由配置控制。
- Standalone 组合完整组件；分布式示例拆分进程，共用 PostgreSQL 和 SeaweedFS，当前面向单机 Compose。
- Git 复制为异步复制，支持受控提升；不提供 Raft 无损自动故障切换。
- 离线备份工具采集完整停止的 Compose 项目，使用本机已有的原镜像 ID 恢复到独立项目；见[生产部署](production-deployment.md)。
- 当前交付 Web，没有桌面本地仓库管理功能。

具体组件关系见[整体架构](architecture/overview.md)，实际验证范围见[验证状态](verification.md)。

</div>
