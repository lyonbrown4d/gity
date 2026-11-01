# Rust 迁移与验证

## 迁移边界

当前分支为 `codex/rust-full-migration`。Rust 服务端、Runner 和 Web 替换已完成，旧 Go/React 实现及构建遗留已清理，原版代表页面的视觉与功能验收已完成。主分支仍保留迁移前实现；合并和发布仍须按仓库流程完成。

当前为 Web 交付：Rust/Axum/SeaORM 服务端、Leptos/WASM Web、独立 Runner、Git 钩子和搜索 CLI。常规 Git 操作通过 git2/libgit2；原生 Git 承担智能 HTTP/SSH Git 协议和必要维护；SSH 由 russh 独立监听可配置端口。

迁移以现有业务、权限和实际页面为基准；数据结构以最新 Rust schema 为准，不接管旧 Go 库。认证验证密码、轮换刷新令牌，并只保存令牌摘要。CI 只支持 `.gity-ci.yml` YAML，旧 DSL 不迁移。

## 已执行的验证

以下结果适用于 Linux x86_64 本机环境，不代表其他架构、正式发布或完整生产恢复流程已验证：

- 最新初始 schema 在 SQLite、PostgreSQL、MySQL 各保留 46 张业务表；表结构、约束和索引已与重整前实现逐项比较。明确变化为实际用户状态字段及 MySQL 会话时间微秒精度。
- 三种数据库统一使用四个初始迁移文件，SSH 功能另通过 V0005 添加部署密钥指纹索引。初始化只接受空库或具有匹配迁移记录的 Rust 数据库；不识别或转换旧 Go 表结构。核心测试覆盖重复执行、历史校验与未受管数据库拒绝。
- 用户分页、禁用后的登录/令牌拦截、会话撤销、密码原子更新及删除账户后的凭据清理，四项回归测试在 SQLite、PostgreSQL、MySQL 均通过。
- 已删除 481 个 Go 源码、234 个旧 SQL、旧 React 前端目录及 Go/Node 构建发布配置。清理后 workspace 测试 149 项通过、0 项失败、1 项忽略；默认忽略的 Runner Docker 用例单独运行 1 项通过。SQLite schema 2 项及 PostgreSQL、MySQL 当前 schema 测试各 2 项通过；服务端 45 项历史回归在 SQLite、PostgreSQL、MySQL 均通过。
- 清理后的 `cargo xtask source-check`、`cargo xtask vendor-check`（6 项）、`cargo fmt --all -- --check`、全 workspace native all-targets 严格 Clippy 和 WASM 严格 Clippy 均通过。统一 `cargo xtask release` 最终运行通过，构建四个原生应用及 Web；Trunk optimized WASM 为 8.3 MiB。
- 冻结源码构建的 `gity-rust:blackbox` 和 `gity-runner:blackbox` 镜像均已在新建测试容器中重建并完整复验通过。server 镜像 ID：`sha256:422ab7fb5fc447dc523c6ec11cda2bcf4d58c54b6376089efa2bd5e26fa297f1`；Runner 镜像 ID：`sha256:b2f5690cc7cee53d9c6f9d121ae90c354b1c857f06fffc9c334513dc7e4a503b`。
- 集成环境使用 PostgreSQL 18、SeaweedFS 4.51；11 项功能验收和 3 项 SSH 负向隔离检查通过。S3 实测覆盖 9 MiB 对象、Range、分段上传及对象字节校验；真实 Runner 容器任务为 `Succeeded`，并核对 artifact 字节。所有测试使用隔离环境与卷，未触碰现存用户持久卷。
- 真实 Git/SSH 客户端已验证双克隆间 clone、push、pull 与文件一致性、密钥隔离和分支保护。最新独立 Release CLI 双克隆 push/pull 通过。
- OTLP 请求追踪已实际送入 Collector/Tempo。最终 Release 构建包括四个原生应用和 Web。
- 已完成原版代表页面视觉对照及功能核验，移动 Drawer 的关闭按钮、Escape、遮罩、路由及桌面/移动切换均通过浏览器验收。遮罩 80% 不透明度、Drawer 实际宽度 238.39 px（原版 237.39 px）及交互检查通过；最终 Host/Trunk CSS 构建通过。5 项浏览器检查覆盖真实服务 `18080` 登录、项目、Monaco 编辑器及本地 5 个 WASM 编辑器资源；无 pageerror 或 vendor 加载失败。Rust Releases/Runners 浏览器验收覆盖 Release 创建、修改、删除，asset link 创建、读取、删除（接口不支持更新），tag 创建、删除（不支持修改），Runner 注册和撤销、变量遮蔽与更新。流水线浏览器验收覆盖 YAML 正负校验、创建/取消/重试、专属 Runner 领取、日志和产物上传下载。测试资源已清理。
- 压力测试已取消，未执行吞吐量或性能曲线测试。

功能验收后的单次空闲 `docker stats` 快照中，Rust server 为 0.16% CPU、151.7 MiB 内存，限制为 2 CPU/1 GiB；PostgreSQL 为 111.5 MiB，SeaweedFS 为 253 MiB，分别计量。该快照不是压力测试或持续容量测量。

Linux x86_64 本机以外的平台、PowerShell 发布脚本和完整生产备份恢复流程尚未实测。迁移完成不等于已合并、已发布或这些未测项目已通过。

## 合并与发布前仍须处理

1. 将迁移分支推送并通过 `.github/workflows/ci.yml` 的 Linux x86 CI；`.github/workflows/release.yml` 由 tag 触发，负责 Linux amd64/arm64 发布构建。发布工作流通过后再按流程打 tag 发布。
2. 生产备份恢复、其他架构及 PowerShell 发布脚本仍待各自环境验证。

## 重现入口

```sh
cargo xtask source-check
cargo xtask vendor-check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo check -p gity-web --target wasm32-unknown-unknown --locked
cargo xtask release
```

压力测试已取消，不是发布门槛。跨数据库及 S3 命令见[集成环境](../deploy/integration/README.md)。CI YAML 见 [CI 配置](ci-yaml.md)，共享语言资产见[国际化](i18n.md)。
