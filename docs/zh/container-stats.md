<div lang="zh-CN">

[简体中文](container-stats.md) · [English](../en/container-stats.md)

# 容器统计信息采样

`cargo run -p xtask -- container-stats --container <id-or-name> --duration-seconds 30 --label idle --output /tmp/container-stats.jsonl` 会将 Docker CLI 流式输出的 `docker stats --format '{{json .}}'` 样本记录为 JSON Lines。每条记录包含 `timestamp_unix`、`label` 和未经修改的 Docker 统计对象（包括 CPU、内存、PID 和网络字段）。输出文件以独占方式创建；如果文件已存在，命令会报错。

采样器会在指定时限结束时停止，并终止、等待它自己启动的 Docker CLI 进程；它不会读取容器环境变量。必须安装 Docker，且当前用户能够访问 Docker daemon。此工具采集的是 Docker stats 视图，不是进程 RSS，也不会推算或报告 RSS。需要进程或 cgroup 内存数据时，请另行采集 `/proc/1/status` 或 cgroup 快照。

可在隔离的短生命周期容器上进行黑盒采样：

```sh
docker run -d --rm --name gity-stats-smoke alpine:3.22 sleep 90
cargo run -p xtask -- container-stats --container gity-stats-smoke --duration-seconds 5 --label smoke --output /tmp/gity-stats-smoke.jsonl
docker stop gity-stats-smoke
```

检查 JSONL 记录，并确认五秒采样窗口结束后 Docker CLI 已退出。每次运行都使用新的输出路径。

</div>
