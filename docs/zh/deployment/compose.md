<div lang="zh-CN">

[简体中文](compose.md) · [English](../../en/deployment/compose.md)

# Compose 组件部署

`deploy/components/` 包含两个以单台 Docker 主机为目标的部署文件：

- `standalone.compose.yaml` 运行完整 Gity 单体应用，并启动单独的 Docker-in-Docker（DinD）服务供 Runner 使用。应用内的 Runner 模块会监督 Runner 子进程。
- `distributed.compose.yaml` 拆分 API、Git server、Gateway、Registry、Worker 和两个 Git storage node，使用 PostgreSQL 与 SeaweedFS。分布式 Runner 和 DinD 仅由 `runner` profile 启动。

启动机制见 [Standalone](../architecture/standalone.md) 和[分布式组件](../architecture/distributed.md)。

这两个布局均不是多主机部署。分布式 Git 复制为主节点确认后异步复制，只有受控提升；它不提供 Raft 自动故障切换。部署边界和备份范围见[部署说明](../production-deployment.md)。

## 准备环境

在仓库根目录执行。选择 Debian 或 Alpine 环境模板，复制为本地忽略的 `.env`，再为数据库、RPC 和 S3 设置独立值。模板中的 `replace-...` 都是占位符，不能直接用于部署。

```sh
cp deploy/components/.env.example deploy/components/.env
# 或选择 Alpine：
cp deploy/components/.env.alpine.example deploy/components/.env
chmod 600 deploy/components/.env
```

使用 `openssl rand -hex 32` 生成独立的 `POSTGRES_PASSWORD`、`AUTHORITY_PEER_TOKEN`、两个 Git node peer token 和 SeaweedFS access/secret key。Docker Hub 上游账号和令牌只在私有镜像仓库需要认证时填写，必须同时提供或同时留空。`GITY_DISTRIBUTED_RUNNER_TOKEN` 在 Runner 注册前不用填写真实值。

`GITY_OUTBOUND_HTTP_PROXY`、`GITY_OUTBOUND_HTTPS_PROXY` 和 `GITY_OUTBOUND_NO_PROXY` 可为 Gity 应用及 DinD 容器设置标准 `HTTP_PROXY`、`HTTPS_PROXY` 和 `NO_PROXY` 环境变量。两个环境模板默认不启用代理；需要代理时填写对应代理 URL。默认 `NO_PROXY` 包含本机地址和 Compose 内部服务名，自定义时请保留需要直连的地址。代理地址必须能从容器访问，不能填写宿主机回环地址。镜像构建的出站代理独立使用 Docker 标准 `HTTP_PROXY` / `HTTPS_PROXY` build args。

Compose 任务容器和 Docker daemon 必须在同一个绝对路径看到 Runner 工作目录。模板默认使用 `/tmp/gity-runner-work`；在启动前创建该目录并交给容器 UID/GID `10001`：

```sh
sudo install -d -o 10001 -g 10001 /tmp/gity-runner-work
```

### 生成 TLS 和秘密文件

`generate-tls.sh` 会创建本地 CA、API 与两个 Git node 服务证书，以及 API、Git server、Worker 的客户端证书。证书 SAN 包含 Compose 服务 DNS 名称。该脚本可通过 `GITY_TLS_DIR` 指定输出目录。`prepare-secrets.sh` 从进程环境读取 `.env` 值，生成 Git node token、SeaweedFS S3 身份 JSON 和 Angos S3/镜像配置，可通过 `GITY_SECRETS_DIR` 指定输出目录。设置这些变量时，应使用 Compose 可以绑定挂载的绝对目录。

```sh
set -a
. deploy/components/.env
set +a
export GITY_TLS_DIR="$PWD/deploy/components/tls"
export GITY_SECRETS_DIR="$PWD/deploy/components/secrets"
sh deploy/components/generate-tls.sh
sh deploy/components/prepare-secrets.sh
sudo chown -R 10001:10001 "$GITY_TLS_DIR" "$GITY_SECRETS_DIR"
sudo chmod 0700 "$GITY_TLS_DIR" "$GITY_SECRETS_DIR"
sudo chmod 0600 "$GITY_TLS_DIR"/*.key
sudo chmod 0644 "$GITY_TLS_DIR"/*.crt
sudo chmod 0400 "$GITY_SECRETS_DIR"/*
```

服务容器以 UID/GID `10001` 读取这些文件。私钥保持 `0600`，S3、Angos 和 node token 文件保持 `0400`，两个目录为 `0700`。秘密目录由 Compose 只读挂载；不要将生成文件提交到 Git。证书由本地 CA 签发，仅适合本机 Compose 使用；生产环境应替换为组织管理的 CA 和证书。

修改环境模板后，可先检查 Compose 展开结果。重定向可避免把包含环境值的配置打印到终端：

```sh
docker compose --env-file deploy/components/.env \
  -f deploy/components/standalone.compose.yaml config > /dev/null
docker compose --env-file deploy/components/.env \
  -f deploy/components/distributed.compose.yaml config > /dev/null
```

## 启动 standalone

Standalone 使用配置文件中的 SQLite 与本地对象目录；Runner 通过同一 Compose 项目中的 DinD 服务执行容器任务。它发布 HTTP `8080`、Git SSH `2222` 和 OCI `5000`：

```sh
docker compose \
  --env-file deploy/components/.env \
  -f deploy/components/standalone.compose.yaml \
  up -d --build
docker compose \
  --env-file deploy/components/.env \
  -f deploy/components/standalone.compose.yaml \
  ps
```

访问 `http://localhost:8080`；健康检查位于 `/api/health`。Docker daemon 使用 TLS，Docker socket 不挂入 Gity 容器。DinD 要求 privileged 容器，需在隔离的 Docker 主机运行。

## 启动分布式组件

基础栈启动时不要启用 `runner` profile。Compose 会等待 PostgreSQL、SeaweedFS 和 `seaweed-buckets` 就绪，再运行一次迁移并启动服务。初始化任务使用同一份 S3 身份创建 `gity-artifacts` 与 `gity-oci` 桶：

```sh
docker compose \
  --env-file deploy/components/.env \
  -f deploy/components/distributed.compose.yaml \
  up -d --build
docker compose \
  --env-file deploy/components/.env \
  -f deploy/components/distributed.compose.yaml \
  ps
```

Gateway 对外发布 HTTP `8080` 和 Git SSH `2222`；OCI 发布在 `5000`。软件包客户端使用 Gateway 下的 `/api/v1/projects/{id}/packages/...` 或 `/api/v1/repos/{id}/packages/...` 路径。Gateway 将软件包请求转发至 Registry 内部 `8082`；Registry 的 OCI 服务监听内部 `8081`。Authority RPC `9100` 与 Git storage RPC `9101` 不发布到宿主机。

Gateway 默认按 HTTP 部署。若外部 TLS 代理终止 HTTPS，将 `deploy/components/config/gateway.toml` 中的 `public_scheme` 改为 `https`，让 Gateway 转发正确的公开协议；`Host` 会随请求转发。

### 两阶段启动分布式 Runner

先在 Gity 项目设置中注册 Runner，并取得项目范围 Runner token。令牌只在下面这次命令的交互环境里提供；读取时不回显，也不写入 `.env`：

```sh
read -r -s -p "Runner token: " GITY_DISTRIBUTED_RUNNER_TOKEN
printf '\n'
export GITY_DISTRIBUTED_RUNNER_TOKEN
docker compose \
  --env-file deploy/components/.env \
  -f deploy/components/distributed.compose.yaml \
  --profile runner up -d --build docker runner
unset GITY_DISTRIBUTED_RUNNER_TOKEN
```

命令使用 Bash 的静默输入。Runner 连接 `http://gateway:8080`，通过 TLS 连接 DinD 的 `tcp://docker:2376`。Runner 与 DinD 使用同一绝对工作路径挂载，源码和脚本才能在 Docker daemon 所在容器中正确找到。

## 检查与停止

Compose 健康检查只表示对应进程的就绪探针通过。启动后还应检查 `http://localhost:8080/api/health`，并用实际 Git clone/push、包上传/下载和 OCI push/pull 验证数据路径。Git node 未配置宿主机发布端口。

使用与启动时相同的 `-f` 和 `--env-file` 参数执行 `docker compose down` 可停止服务并保留命名卷。不要使用 `down -v`，它会删除数据库、仓库、对象存储及 Docker 数据卷。

</div>
