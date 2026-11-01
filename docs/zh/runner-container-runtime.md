<div lang="zh-CN">

[简体中文](runner-container-runtime.md) · [English](../en/runner-container-runtime.md)

# Runner 执行与容器运行

本文对应 `main` 中的 Rust Runner。流水线 YAML、任务依赖和变量规则见 [CI YAML](ci-yaml.md)。服务端负责调度；独立的 `gity-runner` 进程领取并执行项目任务。

## 启动与配置

Runner 启动时读取 `.env`。命令行参数覆盖同名环境变量；项目 Runner 令牌由服务端注册接口生成，通过 `GITY_RUNNER__TOKEN` 传入。

```sh
GITY_RUNNER__TOKEN='project-runner-token' cargo run -p gity-runner -- \
  --server http://localhost:8080 \
  --work-root /srv/gity-runner
```

| 参数 | 环境变量 | 默认值与约束 |
| --- | --- | --- |
| `--server` | `GITY_RUNNER__SERVER_URL` | `http://localhost:8080`；HTTP(S)，不能内嵌凭证 |
| `--token` | `GITY_RUNNER__TOKEN` | 必填，项目专属令牌 |
| `--executor` | `GITY_RUNNER__EXECUTOR` | `container`，也支持 `shell` |
| `--container-bin` | `GITY_RUNNER__CONTAINER_BIN` | `docker`，运行 Docker CLI |
| `--image` | `GITY_RUNNER__IMAGE` | `alpine:3.22`；任务 YAML 的 `image` 优先 |
| `--work-root` | `GITY_RUNNER__WORK_ROOT` | `data/runner`；每次任务创建独立临时目录 |
| `--memory-mb` | `GITY_RUNNER__MEMORY_MB` | 1024 MiB；64–262144 |
| `--cpus` | `GITY_RUNNER__CPUS` | 2；1–256 |
| `--pids-limit` | `GITY_RUNNER__PIDS_LIMIT` | 256；16–65536 |
| `--poll-seconds` | 无 | 3 秒；空队列实际等待限制为 1–60 秒 |
| `--once` | 无 | 完成一次领取流程后退出，空队列也会退出 |

参数解析与资源范围校验失败时，Runner 在领取任务前退出。持续运行时，领取或执行错误会输出诊断，等待 5 秒后继续轮询。

## 容器执行

`container` 模式需要 Docker CLI 和可访问的 Docker daemon。Runner 使用 `docker run` 创建任务容器，将源码目录挂载到 `/workspace`，将脚本只读挂载到 `/gity-script.sh`，执行 `sh -eu /gity-script.sh`。当前实现没有独立的 Podman、containerd 或 Firecracker 执行器。

在 Unix 上，任务容器使用 Runner 的 UID/GID。容器设置内存、CPU 和进程数量限制，移除 Linux capabilities，并启用 `no-new-privileges`。CI 变量通过权限为 `0600` 的临时 env 文件交给容器；容器变量值必须为单行，不能含 NUL、回车或换行。

Runner 自身若运行在容器内，Docker bind mount 的源路径由宿主机 daemon 解析。工作目录必须以相同的绝对路径出现在 Runner 容器与 Docker 宿主机上。生产 Compose 的可选 `runner` profile 使用 `GITY_RUNNER_WORK_ROOT` 指定宿主机路径，默认 `/srv/gity-runner`；这是 Compose 变量，Runner 进程读取的配置名仍是 `GITY_RUNNER__WORK_ROOT`。部署前确保 UID/GID `10001` 可以写入该目录，并按宿主机 Docker socket 的 GID 设置附加组。

任务容器只接收任务 CI 变量和工作目录。Docker socket 授予 Runner 管理宿主机容器的能力；Runner 进程及该 socket 应放在可信的运行环境中。

## Shell 执行

`--executor shell` 直接在宿主机通过 `sh -eu` 运行脚本，只适合受信任的任务。Runner 清除继承的进程环境，再设置 `PATH`、任务临时 `HOME` 和任务变量；容器资源参数不会限制 shell 进程。

## 源码、日志与任务结束

Runner 下载服务端提供的固定提交 ZIP，再解压到任务工作目录。压缩包最多 256 MiB，展开后最多 512 MiB、100000 个条目；路径跳转、`.git` 元数据和指向工作目录外的符号链接会被拒绝。

Runner 每 15 秒发送心跳，核对服务端返回的任务 ID 和执行次数；任务被重新领取后，旧进程也会停止。Runner 分块上传 stdout/stderr。服务端在持久化日志时掩码项目秘密；Runner 单次任务最多上传 4 MiB 输出，超出部分标记为截断。成功任务才上传显式声明的产物，每个产物必须是工作目录内的普通文件，最大 64 MiB。

任务超时、取消或租约失效后，Runner 终止执行进程；Unix 平台会结束整个进程组，容器模式还调用 `docker rm --force`。正常结束时也回收子进程，并删除任务临时目录。任务状态、日志和产物写入携带领取时的执行次数，服务端拒绝过期执行者继续写入。

## 验证

以下 Rust 测试验证 Runner 二进制的 HTTP 领取、源码下载、脚本执行、日志、产物、成功与失败报告，以及超时和租约撤销。容器用例需要本机 Docker daemon 和 `alpine:3.22` 镜像。

```sh
cargo test -p gity-runner
cargo test -p gity-runner --test lifecycle docker_binary -- --ignored --nocapture
```

本次工作区已在 Linux 上运行 shell 与真实 Docker 用例，4 个执行流程测试通过。Windows 进程树清理和其他容器运行时没有运行验收。

实现依据：[命令行入口](https://github.com/lyonbrown4d/gity/blob/main/apps/gity-runner/src/main.rs)、[执行与回收](https://github.com/lyonbrown4d/gity/blob/main/apps/gity-runner/src/executor.rs)、[容器参数](https://github.com/lyonbrown4d/gity/blob/main/apps/gity-runner/src/command.rs)、[资源校验](https://github.com/lyonbrown4d/gity/blob/main/apps/gity-runner/src/resources.rs) 和 [集成测试](https://github.com/lyonbrown4d/gity/blob/main/apps/gity-runner/tests/lifecycle.rs)。

</div>
