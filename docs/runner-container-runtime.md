# Runner Container Runtime

The runner supports multiple execution modes and prefers API-based runtime clients.

## Execution Model

- `GITY_RUNNER_EXECUTION_MODE` (or payload `execution_mode`) chooses runtime explicitly.
- When empty, runner falls back to:
  - host mode when no script image is configured;
  - configured `GITY_RUNNER_CONTAINER_RUNTIME` when an image is configured;
  - `docker` as default container runtime.

Supported values:

- `host`
- `docker`
- `podman`
- `containerd`
- `firecracker`

## Docker / Podman / containerd (implemented)

`docker`、`podman`、`containerd` 使用独立的 runner 路径（`dockerScriptRunner`、`podmanScriptRunner`、`containerdScriptRunner`），
当前共用兼容的 container runtime API 客户端（基于 `github.com/docker/docker`）实现：

- create container via `github.com/docker/docker` client API,
- ensure image exists and pull when missing,
- bind mount workspace,
- capture stdout/stderr via attachment stream.

Runtime endpoint can be set with `GITY_RUNNER_CONTAINER_RUNTIME_ENDPOINT`.

## containerd (API mode)

`containerd` 已接入与 `docker`/`podman` 相同的 API 执行链路。
当前实现会通过 `GITY_RUNNER_CONTAINER_RUNTIME_ENDPOINT` 连接对应 containerd socket，
并以容器创建/运行/等待流程执行脚本。

Required values:

- `containerd`: `GITY_RUNNER_CONTAINER_RUNTIME_ENDPOINT`

## Firecracker (reserved)

`firecracker` 目前仍为保留状态，保持清晰错误返回，便于后续在独立 VM 层实现中逐步补齐。  
相关参数：`GITY_RUNNER_FIRECRACKER_SOCKET`
