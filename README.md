# Gity

Gity 是使用 Rust 构建的 Git 托管与协作平台，提供仓库托管、合并请求、CI、Runner、包仓库和 OCI 镜像缓存。当前代码以 `main` 上的 Rust 实现为准。旧 Go 与 React 实现已从当前源码移除；其迁移过程见[迁移记录](docs/rust-migration.md)。

## 应用与共享库

- `apps/gity-standalone`：组合完整平台模块的单体部署入口。内置 Runner 模块会监督一个独立 Runner 子进程；Runner 不在 API 请求进程中执行流水线脚本。
- `apps/gity-api`、`apps/gity-git-server`、`apps/gity-git-node`、`apps/gity-gateway`、`apps/gity-registry`、`apps/gity-worker`：可分别部署的 API、Git 协作、Git 存储节点、入口网关、OCI 与软件包仓库、后台 Worker。
- `apps/gity-runner`、`apps/gity-git-hook`、`apps/gity-search-index`：流水线执行器、原生 Git 更新钩子和索引管理工具。
- `apps/gity-web`：使用 Leptos 与 WebAssembly 编写的 Web 前端，通过 Cargo 和 Trunk 构建。
- `crates/gity-platform`：API 业务和 standalone 组合入口；`crates/gity-package-registry`：独立软件包协议服务；`crates/gity-git-cluster` 与 `crates/gity-git-node`：Git 集群调度和存储节点。
- `crates/gity-core`、`crates/gity-entities`、`crates/gity-ids`：配置、领域类型、SeaORM 实体和 Snowflake ID；`crates/gity-observability`：各角色共享的 JSON/文本日志、Prometheus 指标和可选 OTLP 追踪。
- `migrations`：SQLite、PostgreSQL、MySQL 的版本化迁移；`xtask`：仓库构建、依赖资产校验和源码检查。

Monaco 与 Milkdown 以带许可证和校验值的本地静态资产分发。生产页面不依赖 CDN，也不使用 Node 构建链。

## 本机运行

需要 `rust-toolchain.toml` 指定的 Rust 工具链、原生 Git 和 Trunk：

```sh
rustup show
cargo install trunk --version 0.21.14 --locked
cargo xtask web
cargo run -p gity-standalone -- --config config/gity.example.yaml
```

默认打开 `http://localhost:8080`。Swagger UI 位于 `/docs`，OpenAPI JSON 位于 `/openapi.json`，HTTP 健康检查位于 `/api/health`。示例配置使用本地 SQLite 和文件对象存储。服务端支持 TOML/YAML 配置、环境变量和 CLI 参数；优先级为代码默认值、配置文件、环境变量、CLI 参数。

也可以把配置文件作为参数传给其他应用，例如 `gity-api`、`gity-git-server`、`gity-worker`。各应用的职责和默认配置见 `apps/` 与 `deploy/components/config/`。

## 部署

仓库当前的 Compose 部署入口是 [deploy/components](deploy/components/README.md)：提供完整 standalone 和单机 Compose 分布式组件两种布局，支持 Debian 与 Alpine 基础镜像。Standalone 默认使用 SQLite 和本地对象目录；分布式布局使用 PostgreSQL、SeaweedFS S3、两个 Git 存储节点和可选 Runner。分布式 Git 复制为主节点确认后异步跟进；没有 Raft 自动故障切换。详细准备、启动和备份边界见[部署说明](docs/production-deployment.md)。

`docker-compose.prod.yaml` 与 `build/package/Dockerfile` 是旧部署入口，不作为当前组件部署指南。各角色镜像构建参数与当前验证边界见[镜像变体](build/package/variants.md)。本轮按用户决策以静态检查通过收口，完整镜像黑盒矩阵尚未完成；具体结果见[重构与验证记录](docs/component-refactor-validation-2026-11-01.md)。

数据库以当前 Rust schema 为准，不接管历史 Go 数据库；后续 Rust schema 通过版本化迁移维护。

## 常用验证

```sh
cargo xtask source-check
cargo xtask vendor-check
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo check -p gity-web --target wasm32-unknown-unknown --locked
cargo xtask release
```

原版代表页面的视觉与交互对照以及 Rust 迁移记录见[迁移说明](docs/rust-migration.md)。压力测试已取消；当前资料没有吞吐量或性能曲线数据。上述验证命令用于复现；本轮实际执行范围见[验证记录](docs/component-refactor-validation-2026-11-01.md)。

## CI 与语言资产

CI 配置文件为 `.gity-ci.yml`，采用 YAML，不使用自定义 DSL；示例见 [CI YAML](docs/ci-yaml.md)。Web 和服务端共用 `crates/gity-i18n/locales` 中的翻译资产。Web 保存语言偏好并发送 `Accept-Language`，服务端按请求选择语言；详见[国际化说明](docs/i18n.md)。
