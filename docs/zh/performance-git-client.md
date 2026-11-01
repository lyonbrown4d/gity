<div lang="zh-CN">

[简体中文](performance-git-client.md) · [English](../en/performance-git-client.md)

# Git 客户端压力测试

k6 工作负载覆盖 REST API。Git clone/fetch/push 需要使用真实 Git 客户端，因为 Smart HTTP 通过 `info/refs`、`git-upload-pack` 和 `git-receive-pack` 执行 Git 协议 RPC。

`xtask git-stress` 命令会通过 API 创建测试项目，生成具备 `read_repository` 和 `write_repository` 权限的项目访问令牌，然后并发运行真实的 `git` 子进程。

## 对黑盒环境运行

```bash
cargo run -p xtask -- git-stress --build --workers 4 --iterations 24
```

该命令会启动 `docker-compose.blackbox.yaml`，并在测试结束后保持服务栈运行。默认会清理本地临时 Git 工作树。对于命令创建的项目，测试结束后会撤销访问令牌并将项目标记为待删除。生成的账号和组织会保留在测试数据库中，供数据库生命周期清理流程处理。

## 对现有服务运行

```bash
cargo run -p xtask -- git-stress --no-compose --base-url http://127.0.0.1:18080 --workers 8 --iterations 80
```

## 工作负载模式

```bash
cargo run -p xtask -- git-stress --mode mixed --workers 8 --iterations 100
cargo run -p xtask -- git-stress --mode clone --workers 4 --iterations 20
cargo run -p xtask -- git-stress --mode fetch --workers 16 --iterations 200
cargo run -p xtask -- git-stress --mode push --workers 4 --iterations 40
cargo run -p xtask -- git-stress --mode ls-remote --workers 32 --iterations 500
```

mixed 模式的默认占比为 `clone=20`、`fetch=45`、`push=25` 和 `ls-remote=10`。可使用 `--clone-percent`、`--fetch-percent`、`--push-percent` 和 `--ls-remote-percent` 覆盖。

## 使用现有仓库

如果已有仓库 URL 和令牌：

```bash
GITY_GIT_PERF_TOKEN=<token> cargo run -p xtask -- git-stress --no-compose --project-url http://127.0.0.1:18080/group/project.git --workers 8 --iterations 80
```

命令从 `GITY_GIT_PERF_TOKEN` 读取令牌，并通过临时 askpass 辅助程序将令牌提供给 Git。可用 `--token-username` 覆盖用户名。令牌不会写入仓库 URL 或 Git 参数。使用现有仓库模式不会删除指定项目或其远端分支。askpass 辅助程序需要 Unix shell；尚未验证在 Windows 上运行 Git 压力测试。

## 代理与凭据辅助程序

目标为 localhost 时，运行器会让子进程绕过 Git/HTTP 代理，并清除 Git 凭据辅助程序配置。这样黑盒测试不会受机器级代理设置（如 `HTTP_PROXY`）或 Git Credential Manager 配置影响。

Git Smart HTTP 认证失败时，服务端必须返回 `WWW-Authenticate: Basic`。访问私有仓库时，Git 客户端会先收到此质询，再尝试使用 URL 中的凭据重试。

## 服务端 Git RPC 并发容量

服务端通过 `git.max_concurrent_rpc`（`GITY_GIT__MAX_CONCURRENT_RPC`）限制并发运行的原生 Git Smart HTTP 子进程。默认值为 `32`；设为 `0` 可关闭限制。只有主机具备足够的 CPU、文件描述符和仓库 I/O 容量时才应调高该值。如果 API 与 Git 混合流量开始出现网关错误或长尾延迟，则应调低该值。

## 指标

运行器会按操作打印次数、失败数、平均值、p50、p95 和最大值：

- `setup.init`、`setup.commit`、`setup.push`、`setup.clone`。
- `ls-remote`：检查引用通告和读取。
- `clone`：全新客户端克隆流程。
- `fetch`：增量 `git-upload-pack` 流程。
- `push`：`git-receive-pack` 流程。

`--keep-workdir` 会保留本地工作树以便调试。无论是否设置此选项，命令自行创建项目时都会请求删除项目并撤销测试令牌。推送到现有外部项目的分支会保留。

</div>
