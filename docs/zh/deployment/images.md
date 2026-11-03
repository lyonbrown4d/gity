<div lang="zh-CN">

[简体中文](images.md) · [English](../../en/deployment/images.md)

# 容器镜像变体

[`Dockerfile.variants`](https://github.com/lyonbrown4d/gity/blob/main/build/package/Dockerfile.variants) 为每个应用角色构建独立镜像。默认使用 `rust:1.99.1-bookworm` 编译、`debian:bookworm-slim` 运行；Alpine 组合使用 `rust:1.99.1-alpine` 和 `alpine:3.22`，生成 musl 二进制。Debian 与 Alpine 的构建器和运行时应成对设置。

| `GITY_ROLE` / target | 镜像内容 |
| --- | --- |
| `standalone` | 完整 standalone 应用、静态 Web、Git hook、Runner 和 Docker CLI、Angos OCI 引擎。 |
| `api` | API 应用与静态 Web。 |
| `git` | Git 协作 HTTP/SSH 服务、静态 Web和 Git hook。 |
| `git-node` | Git 存储节点和 Git hook。 |
| `gateway` | HTTP/SSH 入口网关，Web 请求由 API 服务提供。 |
| `registry` | OCI 镜像网关、软件包协议服务与 Angos OCI 引擎。 |
| `runner` | 独立 Runner 与 Docker CLI。 |
| `worker` | 后台 Worker。 |

所有 Gity 运行时容器以 UID/GID `10001` 运行。各应用角色使用共享 `gity-observability` 库；运行镜像只包含该角色所需的程序和静态资源。Angos 使用固定 digest 的上游二进制，不在此 Dockerfile 中构建 Angos 源码。

## 本地构建

在仓库根目录使用 BuildKit 构建当前主机架构的 Debian standalone 镜像：

```sh
docker buildx build --load \
  -f build/package/Dockerfile.variants \
  --build-arg GITY_ROLE=standalone \
  --build-arg GITY_BASE=debian \
  --build-arg BUILDER_IMAGE=rust:1.99.1-bookworm \
  --build-arg RUNTIME_IMAGE=debian:bookworm-slim \
  --target standalone \
  --tag gity:local-standalone-debian \
  .
```

构建 Alpine Registry 时，将角色、基础系统和标签改为对应值：

```sh
docker buildx build --load \
  -f build/package/Dockerfile.variants \
  --build-arg GITY_ROLE=registry \
  --build-arg GITY_BASE=alpine \
  --build-arg BUILDER_IMAGE=rust:1.99.1-alpine \
  --build-arg RUNTIME_IMAGE=alpine:3.22 \
  --target registry \
  --tag gity:local-registry-alpine \
  .
```

`GITY_ROLE` 必须与 `--target` 一致。可选参数 `CARGO_JOBS` 默认是 `4`。Compose 已为各角色设置对应的 target 和参数；从 Compose 部署时按[组件部署说明](compose.md)操作。

## 构建缓存与验收边界

Dockerfile 分别缓存系统依赖、Cargo 依赖和 pnpm 全局 store。构建先用不随镜像交付的契约导出工具生成 OpenAPI，再通过 Orval 生成客户端并构建当前 React/Vite 前端。最后各角色单独编译 Rust，将 `dist/web` 嵌入最终二进制；不使用 Trunk 或 Rust Web 前端。Cargo target 缓存按基础系统隔离，并在编译和复制产物期间排他锁定；`CARGO_JOBS` 限制 Rust 编译并发。

Dockerfile 支持 Debian 与 Alpine 角色组合，但配置存在或 `cargo build` 成功都不能证明最终容器可运行。各组合必须分别执行完整镜像黑盒，实际运行范围见[验证记录](../verification.md)。
## 镜像黑盒验收

在仓库根目录执行，每次使用独立且未占用的 Compose 项目名：

```sh
cargo xtask blackbox --mode standalone --base debian \
  --project gity-bb-acceptance-standalone-debian \
  --evidence /tmp/gity-standalone-debian.json
```

分别组合 `--mode standalone|distributed` 与 `--base debian|alpine` 执行四次。工具构建实际镜像并启动独立数据卷，检查原生包客户端、Git HTTP/SSH、CI Runner、OCI/mirror 与权限隔离；分布式用例还检查组件故障和 Git 受控提升。默认结束后删除本次项目及其数据卷，拒绝接管已有项目。宿主机需要空闲的 HTTP 8080、SSH 2222 和 OCI 5000 端口、Docker、Git、SSH 和 OpenSSL。

仅验收当前 Web 资源、API、Git HTTP/SSH、MR 恢复、OCI/mirror 及分布式故障时，加 `--suite protocols`。该范围明确跳过 CI Runner、原生语言包客户端和备份恢复；不等同于默认 `--suite full` 的完整验收。输出证据中的 `suite` 字段记录实际范围。

若依赖下载需要代理，可设置 `GITY_BLACKBOX_DOWNLOAD_PROXY=http://容器可达代理:端口`。工具仅接受不含凭据的 HTTP 代理地址，将其传给本次 Compose 服务及临时包客户端；内部服务保持直连。证据记录镜像 ID、实际系统、UID 和一次容器资源快照，失败时另存脱敏日志；快照不等同于压力测试。

仅在已为同一 `--project`、角色与基础系统构建了当前源码镜像时，才可加 `--no-build` 复用这些标签；此选项仍运行全部功能检查并记录镜像 ID，不能用旧镜像验收新源码。Cargo 下载禁用 HTTP/2 多路复用，以减少代理连接中的流重置；网络受限时可在预构建阶段传入 Docker 标准 `HTTP_PROXY`、`HTTPS_PROXY` build args。

下载代理必须同时能从 Compose 容器和 DIND 内部访问。不要使用 `127.0.0.1`，并避免与 DIND 默认 `172.17.0.0/16` 网段冲突；可使用单独的测试 bridge 地址转发本机代理。SQLite 故障注入使用官方 `python:3.13.2-bookworm` 作为数据库客户端，验收前需预拉取；完整元数据备份使用提供 GNU tar 的 `debian:bookworm-slim` 辅助镜像。两者均为测试/运维工具，不进入应用镜像。

</div>
