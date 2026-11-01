# Rust 部署

本页对应 `main` 的 Rust 部署。Rust 替换、遗留清理及 Linux x86_64 本机验收已完成；正式发布仍以发布工作流的实际结果为准。其他架构、PowerShell 发布脚本和完整生产备份恢复流程尚未实测。

## 构建与启动

`build/package/Dockerfile` 使用 Cargo 构建原生应用，再用 Trunk 构建 Web。最终 server 镜像包含静态 Web、原生 Git 和更新钩子；应用以 UID/GID `10001` 运行。

```sh
cp .env.production.example .env.production
```

填写公开地址 `GITY_BASE_URL`、数据库密码和 S3 密钥后启动。数据库 DSN 使用 URL 格式，因此密码应使用 URL-safe 字符，或对特殊字符执行 URL 编码。

```sh
docker compose --env-file .env.production -f docker-compose.prod.yaml build server
docker compose --env-file .env.production -f docker-compose.prod.yaml up -d server
docker compose --env-file .env.production -f docker-compose.prod.yaml ps -a
```

Compose 先启动 PostgreSQL 和 SeaweedFS，等待数据库就绪及 S3 私有桶可访问，再运行一次 `migrate`；迁移成功后启动 `serve`。Git 仓库与搜索索引共享应用数据卷，数据库和 SeaweedFS 各自使用持久化卷。

HTTP 默认仅绑定宿主机 `127.0.0.1:8080`，由已有反向代理提供 TLS。SeaweedFS 的 S3、Filer、Master 和管理接口不发布到宿主机。单节点 SeaweedFS 仅是此 Compose 的部署形态，不提供多节点容灾。

```sh
curl --fail http://localhost:8080/api/health
curl --fail http://localhost:8080/openapi.json
```

`/api/health` 检查 HTTP 进程，不代表所有外部依赖已就绪；同时检查迁移任务、数据库、对象存储与实际业务读写。

## 命令与配置

| 命令 | 行为 |
| --- | --- |
| `gity-server migrate` | 应用嵌入式版本迁移 |
| `gity-server serve` | 启动 HTTP、流水线调度和搜索刷新 |
| `gity-server standalone` | 先迁移，再启动服务 |
| `gity-server worker` | 独立后台调度，不启动 HTTP |
| `gity-server set-password USERNAME` | 从标准输入读取密码并撤销现有会话 |
| `gity-search-index --project-id ID` | 重建指定项目索引 |
| `gity-search-index --watch` | 按配置持续刷新索引 |

`serve` 已含调度；单实例不需要额外 worker。多进程写入同一数据库时，为每个写入进程设置不同的 `GITY_DATABASE__NODE_ID`（0 至 1023），避免 Snowflake ID 冲突。

日志默认使用结构化 JSON。Prometheus 使用独立监听端口；OTLP 通过 `GITY_OBSERVABILITY__OTLP_ENDPOINT` 启用。生产 Compose 不发布指标端口，需要监控采集时接入受信任网络。

## 数据库初始化与更新

以最新 Rust 数据结构为唯一基准，使用空数据库执行 `gity-server migrate`，或通过 Compose 的一次性 migration 服务初始化。认证表、业务表、索引和约束统一纳入版本化 schema。

不提供 `--adopt-go`，也不识别旧 Go 表结构或旧明文令牌。已有旧版本数据时，应另建数据库；应用不会自动覆盖或转换旧库。不要通过删除迁移记录强行重放初始化 SQL。

后续 Rust 版本通过新增迁移升级，已执行迁移的校验值不允许静默改变。更新前备份数据库、仓库和对象存储；回退时恢复匹配版本的备份，不只更换二进制。

账号密码使用 Argon2，个人和项目令牌只保存摘要，刷新令牌会轮换。`set-password` 是管理员密码管理命令，不承担旧库迁移。

## Runner

Runner 是独立应用，服务端不会直接执行用户脚本。容器 Runner 通过 Docker daemon 启动任务容器；工作目录必须以同一个绝对路径同时存在于宿主机与 Runner 容器，才能正确挂载源码和脚本。

创建工作目录并赋权，再设置项目 Runner 令牌和 Docker socket 的实际组 ID：

```sh
sudo install -d -o 10001 -g 10001 /srv/gity-runner
docker compose --env-file .env.production -f docker-compose.prod.yaml --profile runner up -d runner
```

挂载 Docker socket 赋予 Runner 对该 daemon 的主机管理能力。应使用专用执行主机，不把不受信任项目与服务端数据放在同一执行环境。

## 更新与备份

更新前备份 PostgreSQL、应用数据卷和 SeaweedFS 数据卷，并保留当前镜像标签。不要使用 `docker compose down --volumes` 停止生产服务；该参数会清空持久化数据。

Rust 四应用及 Web 的最终 `cargo xtask release`、Trunk optimized WASM（8.3 MiB）、清理后 workspace 149 passed/0 failed/1 ignored、严格 native/WASM Clippy 和冻结源码 server/Runner 镜像隔离复验均已通过。Linux x86_64 以外发布架构由 tag 触发的 release workflow 验证；生产备份与恢复流程尚未实测。压力测试已取消，未测吞吐量或性能曲线。
