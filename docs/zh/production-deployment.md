<div lang="zh-CN">

[简体中文](production-deployment.md) · [English](../en/production-deployment.md)

# 部署说明

当前仓库的 Compose 部署入口是 [`deploy/components`](deployment/compose.md)，镜像构建方式见[镜像变体](deployment/images.md)。本页概述两种部署形态和运行边界。

这两种 Compose 布局都运行在一台 Docker 主机上。分布式布局把进程和数据卷拆开，但不构成跨主机集群或自动故障切换方案。当前 Debian/Alpine 角色镜像矩阵还需实际构建和黑盒验收；本机 Cargo 检查不能代替镜像验证。

## 选择部署形态

| 形态 | 组件与数据 | 暴露入口 | 适用限制 |
| --- | --- | --- | --- |
| Standalone | 一个 Gity 进程组合 API、Git、OCI、软件包和后台模块；Runner 模块监督独立 Runner 子进程。默认使用 SQLite 和本地对象目录。Docker-in-Docker（DinD）作为单独容器运行。 | HTTP `8080`、Git SSH `2222`、OCI `5000` | Compose 需要 Docker 的 privileged 模式；Runner 容器任务使用单机 DIND 服务。 |
| 分布式组件 | API、Git server、Gateway、Registry、Worker、两个 Git storage node、PostgreSQL、SeaweedFS 和桶初始化任务；Runner 与 DIND 通过 `runner` profile 单独启动。 | Gateway HTTP `8080`、SSH `2222`、OCI `5000`；软件包协议经 Gateway 转发到 Registry 的内部 `8082` | 仍是单机 Compose。Git 由主节点确认写入，副本异步跟进；不提供 Raft 或自动故障切换。 |

分布式 Registry 在 OCI `8081` 与软件包 HTTP `8082` 上提供服务。Gateway 将 `/api/v1/projects/{id}/packages` 和 `/api/v1/repos/{id}/packages` 路径转发到软件包服务，其余主站 HTTP 请求转发到 API。Git server 提供 HTTP/SSH 协作入口，Git node 仅提供内部存储 RPC。

## 分布式数据与故障边界

Git 集群配置 `replicas = 2` 表示总副本数为 2：一个主节点和一个远端存储副本。写入由主节点确认，之后才异步复制；因此主节点确认不代表远端副本已落盘。读取只选择与当前数据代次一致且已追平的副本。

当前只支持受控提升：操作者指定目标节点，系统必须联系旧主节点、将它写入 fencing 状态，再核对目标副本并激活新主节点。旧主节点不可达时，提升会失败；不能把副本进程数、Compose 重启或 Git mirror 当作无损自动故障切换。

分布式 Compose 为两个 Git node 分别挂载 `git-repositories` 和 `git-repositories-2`，并为 PostgreSQL、SeaweedFS、API、Git server、Worker、Registry 和 DIND 数据配置独立命名卷。备份时应同时覆盖数据库、Git 两个节点、对象存储以及需要恢复的各组件状态；只备份 PostgreSQL 不足以恢复仓库和对象。

## 进程、凭据与标识

standalone 中的 Runner 由 `RunnerModule` 启动为独立子进程，服务端负责编排和授权，不直接运行流水线脚本。分布式 Runner 必须在项目中注册后才启动；它使用颁发给该 Runner 的项目范围令牌连接 Gateway。令牌只在启动 Runner profile 的进程环境中提供，不应写入文档、镜像或共享 `.env`。

所有应用容器以 UID/GID `10001` 运行。TLS 私钥应归 UID/GID `10001` 所有并保持 `0600`；部署证书目录应限制为 `0700`。由 `prepare-secrets.sh` 生成的 S3、Angos 和 Git-node 凭据文件应由 UID/GID `10001` 所有并设为 `0400`，目录设为 `0700`。Compose 按角色只读挂载所需文件；Git node 证书须同时允许 `serverAuth` 和 `clientAuth`，副本用自身证书连接源节点。CA 签名私钥不挂入服务容器；宿主机 root 仍可读取并管理它们。

分布式写入进程使用互不相同的 Snowflake node ID：API `1`、Git server `2`、Worker `3`、Registry 软件包存储 `5`。同一数据库上的写入进程不能重复使用 node ID。standalone 的模块共享单个生成器，配置 node ID 为 `1`。

各角色通过共享 `gity-observability` 输出 JSON 或文本结构化日志，并可配置 Prometheus 指标和 OTLP HTTP 追踪。服务端指标默认监听容器回环地址 `127.0.0.1:9090`；Runner 指标默认关闭，开启时默认使用 `127.0.0.1:9091`。Compose 不发布这些端口；外部采集时须配置受信任的容器网络监听地址。

配置文件使用 `[observability]` 段。OTLP 环境变量按角色分别为：

| 角色 | OTLP collector 环境变量 |
| --- | --- |
| Standalone、API、Git server、Worker | `GITY_OBSERVABILITY__OTLP_ENDPOINT` |
| Git node | `GITY_GIT_NODE_OBSERVABILITY__OTLP_ENDPOINT` |
| Gateway | `GITY_GATEWAY_OBSERVABILITY__OTLP_ENDPOINT` |
| Registry | `GITY_REGISTRY_OBSERVABILITY__OTLP_ENDPOINT` |
| Runner | `GITY_RUNNER__OBSERVABILITY__OTLP_ENDPOINT` |

## 更新与恢复

更新前为数据库和所有相关命名卷建立可恢复备份，并保留当前镜像标签。停止服务时使用对应 Compose 文件的 `down`；不要附加 `-v`，否则 Docker 会删除该项目的持久卷。重启后检查 Compose 健康状态，再用实际 HTTP、Git、包协议和 OCI 操作验证业务路径。

当前仓库没有已执行的生产备份恢复演练记录。上述备份范围是由当前卷挂载关系得出的运维要求，不代表已验证端到端恢复成功。镜像矩阵、Git 副本故障行为和完整备份恢复验收结果应以实际运行记录为准。

当前验证范围见[验证状态](verification.md)。

</div>
