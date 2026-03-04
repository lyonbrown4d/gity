# Gix Server Migration Plan

Date: 2026-03-04

## Goal

将 Git 行为完全收敛到 `git` crate，`standalone` 只依赖抽象接口；在不影响现有 HTTP 协议兼容性的前提下，逐步把服务端实现从 `git` 二进制切换到 `gix`。

## Current State

已完成：

- `standalone` 不再直接执行 `git` 命令。
- `init bare`、`list heads refs` 已通过 `git::storage`（`gix`）实现。
- `stateless-rpc`（`upload-pack` / `receive-pack`）已下沉到 `git::rpc`，当前内部仍调用 `git` 二进制。

当前边界：

- `standalone` -> `git` crate（稳定边界）
- `git` crate -> `gix` + `git` CLI（过渡实现）

## Target Architecture

- `git::backend` trait 统一能力：仓库初始化、refs 广告、upload/receive pack、分支同步所需元数据读取。
- 两套后端实现并存：
  - `CliBackend`（当前生产可用，基于 git 二进制）
  - `GixBackend`（逐步补齐）
- 通过配置开关切换实现，支持灰度和回滚。

## Phases

### Phase 1: Interface Stabilization

- 在 `git` crate 定义稳定接口（输入输出 DTO + 错误模型）。
- `standalone` 仅调用接口，不感知具体实现。
- 覆盖现有行为回归测试（尤其 push、权限校验、分支元数据同步）。

Exit criteria:

- `standalone` 中没有 Git 协议细节和命令行逻辑。
- 现有 smoke test 全绿。

### Phase 2: Read Path Gix First

- 优先迁移只读路径：
  - `info/refs` 广告（已是 gix）
  - heads refs 列举（已是 gix）
  - 仓库元信息读取
- 保留 push/fetch 数据通道在 `CliBackend`。

Exit criteria:

- 所有只读路径默认走 `GixBackend`。
- 性能和行为与现网一致（响应头、pkt-line、错误码）。

### Phase 3: Write Path Experiment

- 实验性实现 `receive-pack` 处理链路（feature flag 控制）。
- 并行运行差异比对：
  - 同一请求在 `CliBackend`/`GixBackend` 输出一致性（引用更新、错误语义）。
- 逐步引入 `upload-pack` 的 gix 实现。

Exit criteria:

- 至少一条写路径在受控环境可稳定替换。
- 有可观测指标与自动回退机制。

### Phase 4: Full Switch

- 默认切换到 `GixBackend`。
- `CliBackend` 保留一段观察期，之后降级为应急 fallback 或移除。

Exit criteria:

- 生产稳定运行，无协议兼容性回归。
- 回滚预案演练通过。

## Risks and Mitigations

- 协议兼容风险（smart HTTP 边界行为复杂）：
  - 通过双后端对照 + golden test 缓解。
- 维护成本上升（双实现期）：
  - 明确阶段退出标准，避免长期双轨。
- 性能回归：
  - 加入基准测试（大仓库、并发 push/fetch）。

## Suggested Task Order

1. 在 `git` crate 新增 backend trait + `CliBackend` 适配现有实现。  
2. 将 `standalone::GitBackendService` 改成依赖 backend trait。  
3. 增加协议级回归测试（golden pkt-line、push/fetch smoke）。  
4. 逐步补 `GixBackend` 并按 phase 切流。  
