<div lang="zh-CN">

[简体中文](performance-git-client.md) · [English](../en/performance-git-client.md)

# Git 客户端压力测试

`cargo xtask git-stress` 使用真实 Git 客户端测试 clone、fetch、push 和 ls-remote，支持 HTTP 与 OpenSSH。默认通过 API 创建独立私有项目；HTTP 使用项目访问令牌，SSH 将提供的公钥注册为该项目可推送的部署 key。测试后撤销这项凭据、请求删除项目并注销会话；生成的账号和组织保留到测试数据库清理。令牌和私钥不会写入仓库 URL、诊断日志或 JSON 摘要。

运行环境需要 Git、OpenSSH 和 Unix shell；生成二进制数据使用操作系统随机源 `/dev/urandom`。当前未验证 Windows。k6 的 Git REST/API 压测与这里的原生 Git 协议压测分别计量。

## 连接已有服务

HTTP 是默认传输方式。下面创建并测试一个独立项目：

```bash
cargo xtask git-stress --no-compose \
  --base-url http://127.0.0.1:18080 \
  --workers 8 --iterations 80
```

SSH 必须提供可非交互读取的专用私钥和已经核实的主机公钥文件。先从环境维护者获取主机公钥，再写入 known_hosts；运行器不会自动接受陌生主机。默认 SSH 端口为 `2222`，主机名取自 `--base-url`，也可用 `--ssh-host`、`--ssh-port` 指定。

```bash
ssh-keygen -q -t ed25519 -N '' -f /tmp/gity-perf-key
# 文件内容示例：[127.0.0.1]:2222 ssh-ed25519 <已核实的服务端主机公钥>
chmod 600 /tmp/gity-perf-key /tmp/gity-perf-known_hosts
cargo xtask git-stress --no-compose --transport ssh \
  --base-url http://127.0.0.1:18080 --ssh-port 2222 \
  --ssh-key /tmp/gity-perf-key --ssh-known-hosts /tmp/gity-perf-known_hosts \
  --workers 8 --iterations 80
```

SSH 使用固定的 `git` 用户、严格主机检查和指定的身份文件，忽略机器级 SSH 配置与全局 known_hosts。文件路径通过固定包装程序的独立参数传入，不拼接为可执行 shell 文本。

## 固定窗口、数据大小和克隆深度

没有 `--duration-seconds` 时，`--iterations` 是所有 worker 合计的操作次数。有该选项时，各 worker 在共同的固定窗口内循环执行操作，完成一次才领取下一次，不按目标请求率发包。窗口到期后停止发起新操作，等待已开始的操作完成，因此实际测量时间可能长于请求窗口。`--timeout-ms` 限制每个 Git 子进程，默认 `60000`。

以下配置每次提交重写一个 1 MiB 的随机文件，形成 16 次提交；工作树约为 1 MiB，完整随机历史约为 16 MiB，另有少量 README 和 Git 元数据。

```bash
cargo xtask git-stress --no-compose --transport ssh \
  --base-url http://127.0.0.1:18080 \
  --ssh-key /tmp/gity-perf-key --ssh-known-hosts /tmp/gity-perf-known_hosts \
  --workers 8 --duration-seconds 60 --mode mixed \
  --seed-files 1 --bytes-per-file 1048576 --seed-commits 16 \
  --clone-depth 0 --summary target/git-stress-ssh.json
```

`--seed-files` 默认 `8`；不指定 `--bytes-per-file` 时沿用小文本数据，`--seed-commits` 默认 `1`。指定字节数后，每次提交都生成全新的操作系统随机数据，避免重复填充被 Git 大幅压缩。8 MiB 工作树、4 次提交可用 `--seed-files 1 --bytes-per-file 8388608 --seed-commits 4`。

测量阶段的 clone 默认 `--clone-depth 1`；`0` 表示完整克隆，正整数表示浅克隆深度。初始 worker 工作树始终完整克隆 main。API 建项目、数据写入、初次推送和初始 worker 克隆全部计入 setup，不计入测量窗口。

## 操作模式

`--mode` 支持 `mixed`、`clone`、`fetch`、`push`、`ls-remote`。mixed 默认占比为 clone 20、fetch 45、push 25、ls-remote 10，可通过相应 `--*-percent` 参数修改。

clone 每次使用全新目录；fetch 更新现有 worker 工作树，单独运行时通常主要测量已同步仓库的引用协商。push 在独立的 `perf/<run>/<worker>-<iteration>` 分支写入小提交，不争抢 main。mixed 中的 fetch 可以读取其他 worker 新增的分支。

## JSON 与延迟口径

`--summary PATH` 保存 JSON，终端也打印各操作的 count、fail、avg、p50、p95、p99 和 max，延迟单位为毫秒。分位数使用 nearest-rank，包含失败和超时样本。

摘要的 `measurement.started_unix_seconds` 和 `finished_unix_seconds` 是 worker 共同开始前、最后一个已开始操作结束后的 UTC Unix 秒数，可据此筛选资源样本；`elapsed_seconds` 和吞吐量仍使用单调时钟，避免系统时钟调整影响计量。摘要分别记录 `setup` 与 `measurement` 的实际 `elapsed_seconds`、操作统计和吞吐量，以及 transport、workers、clone_depth、seed_files、bytes_per_file、seed_commits。`throughput_ops_per_second` 是完成的全部操作数除以实际测量时间；`successful_ops_per_second` 只计成功操作。`seed_applied` 表明本次是否真正生成了 seed；已有仓库的 seed 配置不代表其实际数据。测量失败时仍写出摘要，并以非零状态退出。

这些是客户端端到端数据，包含进程启动、本地文件操作和诊断回收；push 还包含 checkout、add、commit，默认 clone 包含临时目录清理。等待使用原生子进程完成通知，超时会终止并回收整个进程组。结果不能直接当作服务端 RPC 延迟。setup 已使仓库、连接路径和文件缓存变暖，结果也不代表冷启动性能。

## 使用已有仓库

HTTP 模式从 `GITY_GIT_PERF_TOKEN` 读取凭据，通过临时 askpass 提供给 Git；可用 `--token-env` 和 `--token-username` 调整：

```bash
GITY_GIT_PERF_TOKEN=<token> cargo xtask git-stress --no-compose \
  --project-url http://127.0.0.1:18080/group/project.git --mode fetch --workers 8 --iterations 80
```

SSH 模式使用已授权给该仓库的 key，不读取 HTTP token，也不调用 API 创建或删除项目：

```bash
cargo xtask git-stress --no-compose --transport ssh \
  --project-url ssh://git@127.0.0.1:2222/group/project.git \
  --ssh-key /tmp/gity-perf-key --ssh-known-hosts /tmp/gity-perf-known_hosts \
  --mode ls-remote --workers 8 --duration-seconds 60 --summary target/git-existing.json
```

使用已有仓库时，调用者负责权限和数据准备；命令保留创建的远端分支。`--keep-workdir` 保留本地工作树，默认仅清理本次生成的目录。localhost 的 HTTP 请求会绕过机器代理和 Git 凭据辅助程序。

## 启动测试 Compose

省略 `--no-compose` 时沿用 `docker-compose.blackbox.yaml`，可加 `--build`，运行后服务栈保持运行。SSH 模式附加临时 Compose 配置以开启并映射指定 SSH 端口；known_hosts 仍须由操作者提前核实准备。生产镜像验收和性能采样应连接已启动的独立环境，避免将构建、其他测试或共享服务的负载混入样本。

</div>
