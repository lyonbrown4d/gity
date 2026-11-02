<div lang="zh-CN">

[简体中文](performance-k6.md) · [English](../en/performance-k6.md)

# k6 性能测试

Git REST 压力场景使用 `perf/k6/git-api.js`。它与真实 Git/SSH 客户端压力测试、功能黑盒和常用业务 API 场景 `common-api.js` 分开。测试会写入数据，应对专用验收环境运行，并在构建和功能检查结束后开始采样。

## Git REST 读写负载

setup 注册独立 owner 和 outsider 账户，各自获得个人 namespace，再在 owner namespace 下创建私有项目。密码使用 k6 `crypto.randomBytes` 生成。项目包含 README 和约 1 MiB 的随机 ASCII 文件；不创建 `.gity-ci.yml`，不把 Runner 任务混入 Git 压力负载。

默认约 20% 的迭代执行文件提交，其余迭代平均选择分支、提交列表、目录树和 blob。每个 VU 每 100 次迭代额外执行一次 outsider 权限检查：

| 接口 | 校验 |
| --- | --- |
| `GET /projects/{id}/repository/branches` | main 保持初始 SHA，所有分支返回合法提交 SHA。 |
| `GET /projects/{id}/repository/commits` | main 提交列表包含种子提交，SHA 格式正确。 |
| `GET /projects/{id}/repository/tree` | 文件路径、类型和大小与种子一致。 |
| `GET /projects/{id}/repository/blob` | 1 MiB 内容大小及 SHA-256 完全一致。 |
| `POST /projects/{id}/file-commits` | 每个 VU 使用自己的分支和每次迭代独有的新文件路径；检查返回 SHA，再按该不可变 SHA 读取并比较内容。 |
| outsider blob 读取 | 私有项目返回预期 `403`。 |

写入比例按迭代计算；每次成功写入还包含一次读回，因此不等于 HTTP 请求数的写入比例。teardown 只把本次项目标记删除，并注销本次两个会话；不会清理其他项目、账户或 namespace。整个临时服务栈由验收操作者结束后清理。setup 中途失败也会尝试清理已创建的本次项目和会话。

## 在现有服务上运行

先创建新的输出目录。以下示例顺序运行 1、8、32 个 VU，每组持续 60 秒，并为每组保留独立 JSON：

```sh
mkdir -p /tmp/gity-k6-results
for vus in 1 8 32; do
  cargo xtask k6 --no-compose --docker-k6 \
    --base-url http://127.0.0.1:18080 \
    --script perf/k6/git-api.js \
    --vus "$vus" --duration 60s --write-percent 20 --p95 5000 \
    --summary-export "/tmp/gity-k6-results/git-api-${vus}.json"
done
```

`--no-compose` 不启动或清理服务。指定实际入口 URL；示例的 `--p95 5000` 是本次运行的阈值，不是性能保证。测试前记录部署形态、基础镜像、数据库、机器资源和版本，避免同时构建镜像或运行其他负载。不同机器或数据规模的结果不能直接视为同一基准。blob 检查会在负载端解析并哈希 1 MiB 内容，应同时观察 k6 的 CPU，避免把负载端瓶颈当作服务端上限。

本机存在 `k6` 时，省略 `--docker-k6` 会优先使用本机版本；否则使用固定官方镜像 `grafana/k6:1.8.2`，不使用 `latest`。该维护版本可在[官方发布列表](https://github.com/grafana/k6/releases/tag/v1.8.2)核对。Linux 默认使用 host 网络；其他平台使用 Docker Desktop 的 `host.docker.internal`，也可通过 `--docker-base-url` 明确指定容器可达地址。

`--summary-export` 要求父目录已存在，目标文件尚未存在。运行器先原子预留目标文件，在独立的私有临时目录中让 k6 生成 JSON，再验证并写入预留文件。Docker 仅获得该输出临时目录的可写挂载，并使用调用用户的 UID/GID；已有结果不会被覆盖。阈值失败时仍保留有效 JSON，启动失败而没有生成结果时移除空占位文件。

## 指标与预期错误

每个稳定接口名都有独立 `git_api_<endpoint>_duration` Trend、`git_api_<endpoint>_unexpected_failure` Rate、`requests` 和 `successes` Counter。Trend 汇总提供 `med`（p50）、p95、p99、min 和 max；动态项目 ID、VU 分支和令牌不作为指标标签。JSON 由 k6 原生 `--summary-export` 输出，保留这些指标及阈值结果。

`git_api_expected_forbidden` 单独统计权限探针；`git_api_expected_conflicts` 仅统计显式共享分支模式允许的 `409`。新脚本使用 k6 [expectedStatuses](https://grafana.com/docs/k6/latest/javascript-api/k6-http/expected-statuses/) 回调处理预期状态，并用专属 Rate 和内容检查识别意外失败；不会修改旧场景的公共 helper。正常读写接口意外返回 `403` 或 `409` 仍失败，不会计作成功吞吐。

默认要求所有契约检查通过、意外失败率为零，并要求启用写入时至少完成一次提交及精确读回。各接口 p95 必须低于配置阈值。可选 `GITY_K6_CONFLICTS=true` 让 VU 共享 `k6-shared` 分支，单独观察冲突；该模式仍要求真实成功写入，不能只测 `409` 返回速度。

## 参数与其他场景

| 环境变量 | Git 脚本默认值 | Rust 运行器选项 |
| --- | --- | --- |
| `GITY_K6_BASE_URL` | `http://127.0.0.1:18080` | `--base-url` |
| `GITY_K6_VUS` | `8` | `--vus` |
| `GITY_K6_DURATION` | `60s` | `--duration`（运行器默认 `30s`，建议明确指定） |
| `GITY_K6_P95_MS` | `1000` | `--p95` |
| `GITY_K6_WRITE_PERCENT` | `20` | `--write-percent` |
| `GITY_K6_CONFLICTS` | `false` | 调用环境传入，本机与 Docker 均支持。 |

直接执行 `k6 run perf/k6/git-api.js` 时，脚本读取上述变量，也兼容已有 `K6_*` 别名；Rust 运行器把类型化 CLI 参数传给脚本。Git 场景未显式指定写入比例时用 `20`，旧 `common-api.js` 仍用 `12`。脚本不会输出密码、令牌或完整会话响应。

旧常用业务 API 场景仍可通过 `--script perf/k6/common-api.js` 选择。省略 `--no-compose` 会使用 `docker-compose.blackbox.yaml` 启动环境并在测试后保留；`--build` 要求启动时构建，不应用于正式采样过程。

</div>
