<div lang="zh-CN">

[简体中文](verification.md) · [English](../en/verification.md)

# 当前验证状态

本页说明当前代码的检查范围。实现、编译、运行测试和镜像验收分别记录；静态检查通过不表示完整运行验收通过。

## 静态检查

- Native workspace Clippy 与 WASM Clippy 使用固定 Rust 工具链通过检查。
- 格式、源码规模与测试分离约束、第三方编辑器资源校验通过。
- 两份环境模板的 standalone / distributed Compose 配置与部署脚本语法检查通过。
- 文档构建及文档工具的定向 Clippy 由当前文档更新执行；目录、链接、锚点和图表资源在生成站点中检查。

Cargo 提示 `proc-macro-error2 v2.0.1` 存在未来 Rust 兼容性风险，当前固定工具链检查通过。

## 运行覆盖

工作区运行记录为 236 项通过、4 项默认忽略；PostgreSQL、MySQL、OCI 上游解析与容器 Runner 四项专项集成检查另行运行通过。

Debian 的八个角色镜像已有构建记录。Alpine 的 standalone、git-node、gateway、registry、git、api、worker 已有构建记录，独立 Runner Alpine 镜像尚未完成本轮构建。

Standalone / distributed × Debian / Alpine 四组完整黑盒均未完成。当前镜像的完整 SSH/HTTP Git、OCI/mirror、分布式故障链路仍需运行确认，执行方式见[镜像黑盒说明](deployment/images.md#镜像黑盒验收)。

未形成稳态 CPU、峰值内存或吞吐量结论；未验证 arm64、多主机网络及生产备份恢复。当前交付按已授权的静态检查范围执行。

</div>
