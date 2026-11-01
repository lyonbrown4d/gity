# Gity

Gity 是轻量 Git 托管与协作平台。`main` 已统一为 Rust 服务端、Runner 和 Web，后续开发以 Rust 主线为准；Linux x86_64 本机验收已完成。

原版代表页面的视觉与功能核对已完成。主分支已清理旧 Go、React 实现和旧构建配置，并完成清理后的 Linux x86_64 本机复验；其他架构、PowerShell 发布脚本及完整生产备份恢复流程尚未实测。

## 技术与目录

- `apps/gity-server`：Axum HTTP 服务、Clap 命令行、FluxDI 依赖装配、Swagger UI。
- `apps/gity-web`：Leptos WebAssembly 页面，Cargo/Trunk 构建。
- `apps/gity-runner`：独立 Runner，支持容器和本机 Shell 执行。
- `apps/gity-git-hook`：原生 Git 更新钩子适配器。
- `apps/gity-search-index`：持久化仓库索引构建工具。
- `crates/gity-core`：类型化配置、数据库迁移、存储、ID 和请求语言上下文。
- `crates/gity-git`：git2/libgit2 仓库操作与原生 Git 流式协议。
- `crates/gity-search`：Tantivy 索引与提交版本校验。
- `crates/gity-i18n`：前后端共享的中英文资产。
- `migrations`：SQLite、PostgreSQL、MySQL 版本化 SQL。
- `xtask`：Rust 构建、第三方资产校验与源码规模检查。

Web 应用逻辑使用 Rust，Monaco 和 Milkdown 作为带许可证与校验值的本地第三方静态资产复用，不依赖生产 CDN 或 Node 构建链。

## 本机运行

需要 `rust-toolchain.toml` 指定的 Rust 工具链、原生 Git 和 Trunk：

```sh
rustup show
cargo install trunk --version 0.21.14 --locked
cargo xtask web
cargo run -p gity-server -- standalone
```

打开 `http://localhost:8080`。Swagger UI 为 `/docs`，OpenAPI JSON 为 `/openapi.json`，进程健康检查为 `/api/health`。

默认使用本地 SQLite 和对象目录。`standalone` 先迁移数据库，再启动 HTTP 服务及后台调度；分开执行时使用 `migrate`、`serve` 和 `worker` 子命令。`serve` 本身也包含后台调度，不需要额外 worker 才能运行流水线。

配置优先级为内置默认值、可选 `config/gity.toml`、`--config` 指定文件、`GITY_*` 环境变量。支持 TOML 和 YAML，示例见 [config/gity.example.yaml](config/gity.example.yaml) 与 [.env.example](.env.example)。

```sh
cargo run -p gity-server -- --config config/gity.example.yaml standalone
```

## Docker 与数据库支持

[集成环境](deploy/integration/README.md)提供 PostgreSQL、MySQL、SeaweedFS S3 及可选 Prometheus、Tempo、Grafana、OTLP Collector。

数据结构以最新 Rust schema 为准，需要空数据库初始化，不提供旧 Go 数据库接管或旧存储格式兼容。

```sh
docker compose up -d
```

生产部署入口见[部署说明](docs/production-deployment.md)。Dockerfile 使用 Rust 多阶段构建，并提供 server、runner、search-index 三个目标。

## 验证

```sh
cargo xtask source-check
cargo xtask vendor-check
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo check -p gity-web --target wasm32-unknown-unknown --locked
cargo xtask release
```

原版代表页面的浏览器视觉与功能对照已完成。压力测试已取消，未执行吞吐量或性能曲线测试。迁移状态及平台边界见[迁移与验证](docs/rust-migration.md)。

## CI 与共享语言资产

流水线只读取 `.gity-ci.yml`，不再使用自定义 DSL。配置示例见 [CI YAML](docs/ci-yaml.md)。

语言资产统一放在 `crates/gity-i18n/locales`。Web 保存用户语言偏好并发送 `Accept-Language`；服务端按请求协商语言，不修改进程全局语言。详细规则见[国际化](docs/i18n.md)。
