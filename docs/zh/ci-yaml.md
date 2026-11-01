<div lang="zh-CN">

[简体中文](ci-yaml.md) · [English](../en/ci-yaml.md)

# CI YAML

Rust 服务端默认读取仓库中的 `.gity-ci.yml`，API 也可提交经过相同校验的内联 YAML。内容必须为 UTF-8，最大 1 MiB；任务放在显式的 `jobs` 映射下，采用 YAML 表达任务。内联配置不能自行声明为受信任的仓库配置。

```yaml
name: build-and-test
stages: [build, test]
image: alpine:3.22
variables:
  BUILD_MODE: release
jobs:
  compile:
    stage: build
    script:
      - mkdir -p output
      - printf '%s\n' "$BUILD_MODE" > output/mode.txt
    artifacts:
      paths: [output/mode.txt]
    timeout_seconds: 300
  verify:
    stage: test
    needs: [compile]
    script: test "$BUILD_MODE" = release
    timeout_seconds: 60
```

`script` 接受字符串或字符串数组。项目、全局及任务级变量按作用域合并；Runner 注入 `CI_PROJECT_PATH`、`CI_COMMIT_SHA`、`CI_COMMIT_REF_NAME`、`CI_JOB_NAME` 等运行上下文。

没有指定 `stages` 时使用 `build`、`test`、`deploy`，任务默认阶段为 `test`。省略 `needs` 表示依赖所有前置阶段任务；`needs: []` 表示无需前置任务。显式依赖必须存在，且属于当前或更早阶段，不允许循环。

配置支持 1 至 200 个任务，超时时间为 1 至 86400 秒。可用 `tags` 匹配 Runner。未知字段会被拒绝，避免拼写错误被静默忽略。

产物路径必须相对工作目录，不能包含路径跳转或通配符。产物通过下载接口单独获取；当前依赖关系控制调度顺序，不承诺将前置任务产物自动解压到后续任务。

## Runner

注册项目 Runner 后，将得到的项目专属令牌交给独立进程：

```sh
GITY_RUNNER__TOKEN='project-runner-token' cargo run -p gity-runner -- --server http://localhost:8080
```

默认使用容器执行，需要 Docker CLI 和可用的 Docker daemon。`--executor shell` 在宿主机执行脚本，仅用于受信任的本地任务。`--once` 处理一次领取流程后退出；完整参数见 `gity-runner --help`。

Runner 流式下载固定提交的源码 ZIP，发送任务心跳、分块日志和产物，再提交完成状态。任务租约限制 Runner 的写权限；租约失效、取消和超时会终止执行进程。

标记为 `protected` 的项目变量只提供给默认分支或匹配分支保护规则的分支，并要求提交仍为该分支当前 tip、YAML 与仓库固定提交的内容一致。客户端传入 `config_source` 不代表配置受信任。标记为 `masked` 的值在日志持久化前被替换，跨日志块的掩码也需要保留。

## 接口

项目流水线、YAML 校验、任务重试、取消、Runner 注册、任务领取和产物接口，以运行服务的 `/docs` 为准。接口定义由实际 Axum 路由及 Utoipa 宏收集，下载 `/openapi.json` 可用于契约检查。

</div>
