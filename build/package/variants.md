# 容器镜像变体

[`Dockerfile.variants`](Dockerfile.variants) 为每个应用角色构建独立镜像。默认使用 `rust:1.99.1-bookworm` 编译、`debian:bookworm-slim` 运行；Alpine 组合使用 `rust:1.99.1-alpine` 和 `alpine:3.22`，生成 musl 二进制。Debian 与 Alpine 的构建器和运行时应成对设置。

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

`GITY_ROLE` 必须与 `--target` 一致。可选参数 `CARGO_JOBS` 默认是 `4`。Compose 已为各角色设置对应的 target 和参数；从 Compose 部署时按[组件部署说明](../../deploy/components/README.md)操作。

## 构建缓存与验收边界

Dockerfile 将系统构建依赖、Rust 源码和 Web 工具拆成可复用阶段。Debian 使用 BuildKit 缓存 apt 索引和软件包；Alpine 缓存 APK 软件包；Cargo registry、Git checkout 和 target 产物按基础系统分别缓存；target 缓存以排他方式覆盖构建和产物复制，防止并行角色或架构覆盖正在打包的文件。wasm-bindgen、wasm-opt 工具按基础系统和架构缓存。`GITY_ROLE` 与 `CARGO_JOBS` 只影响原生 Cargo 构建步骤。每个角色单独运行 Cargo，避免一个角色的 feature 集合并入另一个角色的构建调用。Trunk 和 `wasm32-unknown-unknown` 在复制 Rust 源码前安装，Web 构建不依赖 native role builder。

Dockerfile 支持 Debian 与 Alpine 角色组合，但配置存在或 `cargo build` 成功都不能证明最终容器可运行。本轮以静态检查通过作为提交条件，完整镜像黑盒矩阵未完成。已构建角色及实际运行范围见[验证记录](../../docs/component-refactor-validation-2026-11-01.md)。
## 镜像黑盒验收

在仓库根目录执行，每次使用独立且未占用的 Compose 项目名：

```sh
cargo xtask blackbox --mode standalone --base debian \
  --project gity-bb-acceptance-standalone-debian \
  --evidence /tmp/gity-standalone-debian.json
```

分别组合 `--mode standalone|distributed` 与 `--base debian|alpine` 执行四次。工具构建实际镜像并启动独立数据卷，检查原生包客户端、Git HTTP/SSH、CI Runner、OCI/mirror 与权限隔离；分布式用例还检查组件故障和 Git 受控提升。默认结束后删除本次项目及其数据卷，拒绝接管已有项目。宿主机需要空闲的 HTTP 8080、SSH 2222 和 OCI 5000 端口、Docker、Git、SSH 和 OpenSSL。

若依赖下载需要代理，可设置 `GITY_BLACKBOX_DOWNLOAD_PROXY=http://容器可达代理:端口`。工具仅接受不含凭据的 HTTP 代理地址，将其传给本次 Compose 服务及临时包客户端；内部服务保持直连。证据记录镜像 ID、实际系统、UID 和一次容器资源快照，失败时另存脱敏日志；快照不等同于压力测试。
