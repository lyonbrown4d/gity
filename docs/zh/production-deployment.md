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

## 离线备份

`cargo xtask backup` 为指定的 Compose 项目建立离线快照。先安排停写窗口，等待流水线完成，并协调所有会直接修改宿主机 bind 目录的进程。工具按精确项目标签停止整个项目，包括应用、Runner、Git 节点、PostgreSQL、SeaweedFS 和 DIND；确认容器全部停止后才读取数据。此过程需要停机，不提供热备份或零数据丢失的自动故障切换保证。

前置条件是 Unix 宿主机、Docker Compose v2、可用的 `debian:bookworm-slim` GNU tar 辅助镜像，以及与源项目一致的 Compose 文件和私有环境文件。环境文件或调用环境须提供原部署所需的镜像名称等变量。下面的目录父路径必须已存在，`--output` 指定的目录必须尚未存在：

```sh
cargo xtask backup \
  --project gity-prod \
  --compose-file deploy/components/standalone.compose.yaml \
  --env-file /srv/gity-private/compose.env \
  --output /srv/gity-private/backup-20261102
```

分布式部署把 `--compose-file` 改为 `deploy/components/distributed.compose.yaml`，并使用该项目实际的环境文件。不要把别的项目名称、Compose 文件或环境文件混用。工具拒绝共享或外部卷、未声明的项目卷、匿名卷、卷别名，以及与其他运行中容器共享写入的命名卷；实际挂载与所提供 Compose 配置不一致时也会拒绝。当前支持显式 bind 挂载，遇到 Compose 顶层 `configs` 或 `secrets` 对象会拒绝，而不会遗漏其内容。

快照包含项目全部已创建命名卷；尚未创建的声明卷记录为空卷。内容包括数据库、两个 Git 节点、对象存储、各组件状态、SSH 主机密钥、DIND 数据和 Docker TLS 卷。所有 bind 源也会归档，包括组件配置、TLS 证书/私钥、S3/Angos/Git-node 秘密文件和 Runner 宿主机工作目录。完整的已解析 Compose 配置会保存实际容器环境，因而也包含运行中的 Runner 令牌等秘密。

归档通过辅助容器流式写入宿主机文件，不把整个卷读入内存。辅助容器以 root 只读访问源目录，宿主机归档仍由调用用户创建。备份目录为 `0700`，工具创建的文件为 `0600`；归档保留数字 UID/GID、权限、扩展属性和 ACL，并支持 Docker 数据层的 whiteout 设备。版本化 `manifest.json` 记录卷映射、镜像身份、文件大小和 SHA-256，不保存明文秘密值。包含完整配置和秘密材料的整个备份包都必须按凭据保管。

对官方、特权模式 Docker DinD 的 `/var/lib/docker` 命名卷，工具精确排除 `volumes/backingFsBlockDev` 和 `overlay2/backingFsBlockDev`。这些 quota 节点记录宿主机设备身份；[Docker 上游实现](https://github.com/moby/moby/blob/v28.5.2/quota/projectquota.go#L380-L401)会按当前文件系统重新创建它们，不能把原设备身份还原到新项目。已有归档中的这两个节点仅在确认为空、root 所有的块设备时允许跳过，提取时仍不创建；普通卷、文件 bind、其他设备及 FIFO 继续拒绝。Docker 数据层中字符设备 `0:0` 的空 whiteout 标记仍保留。

默认情况下，成功或停止后的失败都会让源项目保持停止。只有显式提供 `--resume-source` 且快照成功时，工具才重新启动此前运行的容器。需要马上用原端口恢复时保持源项目停止。失败留下的未完成目录不能当作可恢复备份；工具会拒绝缺失、损坏或校验不匹配的包。

## 恢复到新项目

备份包不包含 Docker 镜像、宿主机操作系统或未挂入项目的文件，例如离线保管的 CA 签名私钥。应另外保留原版本镜像；恢复前本机镜像引用必须仍指向清单中的精确 image ID。只保留一个可能被覆盖的镜像标签不够。

恢复要求新的项目名、新的私有材料目录，以及未被占用的目标卷名称。工具先核验整个清单、所有归档大小/校验值和归档路径，再分配恢复资源；不会覆盖源项目或已有目标数据：

```sh
cargo xtask restore \
  --backup /srv/gity-private/backup-20261102 \
  --project gity-restored \
  --secrets-dir /srv/gity-private/restored-assets
```

工具创建目标项目专用卷，将全部 bind 源映射到新私有目录，并输出 `restored-assets/compose.json`。该文件包含秘密，权限为 `0600`；父目录为 `0700`。归档内容恢复原数字 UID/GID、权限、扩展属性和 ACL。项目网络改为目标项目网络，Runner 的宿主机工作路径也同步改写；原端口和服务配置保留。目标卷以显式 external 名称供该新配置引用，仍带目标项目标签，绝不复用源卷。

恢复不会启动服务。确认原项目停止、端口可用后，再明确启动：

```sh
docker compose --project-name gity-restored \
  --file /srv/gity-private/restored-assets/compose.json \
  --profile '*' up --detach --no-build
```

恢复配置使用 `pull_policy: never`，删除构建入口，避免启动时重新构建或拉取不同镜像。恢复失败时会清理本次新建资源；成功后按实际 HTTP、严格主机密钥 SSH clone/push/pull、软件包、LFS/附件和 OCI 读写及权限隔离检查数据。保留原 `known_hosts` 校验，不能通过重新接受主机密钥掩盖 SSH 身份丢失。

## 更新与运行验证

更新前建立可恢复的完整备份并单独保留原镜像。停止服务使用原 Compose 配置的 `down` 时，不要附加 `--volumes`，否则会删除项目持久卷。MR 合并中断的处理见[合并恢复](merge-recovery.md)；恢复数据库与 Git 时必须保留对应的合并操作记录和内部回执引用。

本页命令已按当前 CLI 和 Compose 定义核对；Standalone/分布式 × Debian/Alpine 四种镜像的完整备份恢复黑盒结果仍需实际运行记录支持。源码检查、编译和测试不能替代这些验收。实际结果见[验证状态](verification.md)。

</div>
