<div lang="zh-CN">

[简体中文](performance-k6.md) · [English](../en/performance-k6.md)

# k6 性能测试

Gity 将 k6 API 性能测试与 Docker 健康状态、OpenAPI、Web 冒烟检查及真实 Git 客户端压力测试分开。主要工作负载会创建一个已认证的组织和项目，然后按权重请求常用 API 路径。

## 对黑盒环境运行

```bash
cargo run -p xtask -- k6 --build --vus 8 --duration 30s
```

如果本机未安装 `k6`，运行器会自动改用 Docker 运行 `grafana/k6`。在 Linux 上，它使用 host 网络，使容器能够访问绑定到 loopback 的黑盒端口。在其他平台上，它使用 Docker Desktop 的 `host.docker.internal` 路由；可通过 `--docker-base-url` 显式指定可访问的 URL。也可以强制使用 Docker 运行 k6：

```bash
cargo run -p xtask -- k6 --build --docker-k6 --vus 8 --duration 30s
```

运行器会启动 `docker-compose.blackbox.yaml`，并在测试结束后保持服务栈运行，以便检查生成的数据。

## 对现有服务运行

```bash
GITY_K6_BASE_URL=http://127.0.0.1:18080 GITY_K6_VUS=16 GITY_K6_DURATION=1m k6 run perf/k6/common-api.js
```

也可以使用 Rust xtask 运行器，而不操作 Docker Compose：

```bash
cargo run -p xtask -- k6 --no-compose --base-url http://127.0.0.1:18080 --vus 16 --duration 1m
```

## 可调参数

- `GITY_K6_BASE_URL`: Web 网关 URL，默认值为 `http://127.0.0.1:18080`.
- `GITY_K6_VUS`: 固定的 VU 数量，默认值为 `8`.
- `GITY_K6_DURATION`: 场景持续时间，默认值为 `30s`.
- `GITY_K6_P95_MS`: p95 延迟阈值（毫秒），默认值为 `1000`.
- `GITY_K6_WRITE_PERCENT`: 分配给轻量写入流程的迭代比例，默认值为 `12`.
- `GITY_K6_DEBUG_FAILURES=true`: 打印失败响应正文。

运行器提供类型化 CLI 选项 `--base-url`、`--vus`、`--duration`、`--p95` 和 `--write-percent`。本机可用 `k6` 时会优先使用；否则通过 Docker 运行 `grafana/k6`。

## 覆盖的 API 范围

- 认证、用户、组织、项目、成员和权限。
- 仓库分支、分支保护、提交、目录树、blob、README、搜索和语言统计。
- Issue、评论、受理人、标签、合并请求、审批、参与者、检查和差异。
- Wiki 页面，以及包注册表的 JSON、Generic、PyPI、NuGet 读取路径。
- 流水线、任务、Runner 注册/心跳/领取任务和 CI 变量。
- LFS 管理和审计事件。

脚本为动态路由添加稳定的 k6 `name` 值，以控制指标的基数。

</div>
