# Rust 组件重构与验证记录

本轮按用户最新决策，以重构完成和静态检查通过作为提交条件，不再要求完整镜像黑盒验收。以下结果区分静态检查、已运行测试、镜像构建与尚未完成的运行验证。

## 本轮实现

- `apps/` 提供 standalone、API、Git server、Git node、Gateway、Registry、Worker、Runner 等可部署入口，`crates/` 复用组件实现。
- 各组件导出 FluxDI module，standalone 组合全功能模块并管理生命周期；分布式入口复用相同业务实现，通过内部 RPC 协作。
- 身份、制品、Git 放置与复制等上下文拆分领域和应用边界；新仓储使用 SeaORM 实体和类型化操作。实体 ID 固定采用应用生成的 Snowflake，对外 JSON 主键使用字符串。
- 软件包服务覆盖 Generic、Cargo、npm、PyPI、Maven、NuGet、Go；OCI 接入 Angos，提供受控 mirror 拉取缓存和项目权限检查。
- 提供配置文件、环境变量、显式 CLI 覆盖及默认值；共享结构化日志、Prometheus 和可选 OTLP；保留宏生成 OpenAPI/Swagger 及共享国际化资产。
- 提供 standalone、单机 Compose 分布式部署及 Debian/Alpine 镜像变体；新增实际镜像黑盒工具与 Testcontainers 集成测试支持。

Git 副本在主节点确认后异步跟进。受控提升要求旧主节点可达、完成 fencing 并核对副本；本轮没有实现 Raft 无损自动故障切换。具体边界见[部署说明](production-deployment.md)。

## 最终静态检查

2026-11-01 在 Linux amd64 上执行，全部通过：

| 检查 | 复现命令或范围 |
| --- | --- |
| Native Clippy | `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` |
| WASM Clippy | `cargo clippy -p gity-web --target wasm32-unknown-unknown --locked --offline -- -D warnings` |
| 格式 | `cargo fmt --all --check` |
| 源码约束 | `cargo xtask source-check`，包含文件规模、测试分离及遗留实现约束 |
| 第三方资源 | `cargo xtask vendor-check`，六项编辑器资产校验通过 |
| Compose | 两份环境模板分别解析 standalone、distributed；distributed 包含 Runner profile |
| 部署脚本 | `sh -n deploy/components/generate-tls.sh deploy/components/prepare-secrets.sh` |
| 补丁格式 | `git diff --check` |

Cargo 另提示第三方 `proc-macro-error2 v2.0.1` 存在未来 Rust 兼容性风险；当前固定工具链检查通过。

## 已执行的测试与构建

本轮此前的 workspace 测试结果为 236 项通过、4 项默认忽略；另显式运行并通过 PostgreSQL、MySQL、OCI 上游解析及容器 Runner 四项集成测试。最终收口阶段没有重新执行运行测试。

Debian 的八个角色镜像均曾构建成功。Alpine 的 standalone、git-node、gateway、registry、git、api、worker 镜像构建成功；独立 Runner Alpine 镜像在用户调整验收要求后停止构建，未计为完成。

standalone Debian 诊断轮次已验证基础鉴权、私有项目隔离、文件提交、12 MiB Generic 包读写与 Range、CI Runner，以及 Cargo、npm、PyPI、Maven、NuGet 原生客户端单项。Go 消费夹具的校验文件流程已修正，未完成修正后的运行复验。

## 未完成的验收范围

standalone / distributed × Debian / Alpine 四组完整镜像黑盒均未完成，不将构建或部分用例通过标记为完整运行验收。当前镜像的 SSH/HTTP Git、OCI/mirror 与分布式故障全流程仍需后续运行确认；执行方式见[镜像黑盒说明](../build/package/variants.md#镜像黑盒验收)。

本轮未运行压力测试，未形成稳态 CPU、峰值内存或吞吐量结论；未验证 arm64、多主机网络及生产备份恢复。

本轮专用测试项目的容器、网络、数据卷与临时代理已回收。原保留检查项目 `gity-rust-blackbox-20261101` 及其业务卷保持原样。
