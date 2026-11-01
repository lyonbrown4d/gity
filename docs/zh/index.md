<div lang="zh-CN">

[简体中文](index.md) · [English](../en/index.md)

# Gity 技术手册

Gity 使用 Rust 提供 Git 托管、项目协作、CI、软件包和 OCI 镜像仓库；Web 使用 Leptos 与 WebAssembly。当前交付 Web 应用。

- 理解系统：[整体架构](architecture/overview.md)、[组件职责与依赖](architecture/components.md)、[数据流](architecture/data-flows.md)。
- 理解启动：[Standalone](architecture/standalone.md)、[分布式组件](architecture/distributed.md)。
- 部署系统：[Compose 操作](deployment/compose.md)、[镜像变体](deployment/images.md)、[部署边界](production-deployment.md)。
- 开发和使用：[CI YAML](ci-yaml.md)、[Runner](runner-container-runtime.md)、[Git SSH](git-ssh.md)、[国际化](i18n.md)。
- 确认范围：[产品范围](roadmap.md)、[验证状态](verification.md)、[发布检查](release-checklist.md)。

手册与 `main` 的当前实现同步。现有 Compose 面向单台 Docker 主机；Git 使用异步副本和受控提升，完整镜像验收范围以验证状态为准。文档工具和中英同步规则见[维护说明](contributing-docs.md)。

</div>
