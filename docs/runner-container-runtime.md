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

## Docker / Podman / containerd

These modes use API-based container runtime clients built on the Docker Go SDK:

- create container from image;
- pull image when missing;
- bind-mount workspace;
- capture stdout/stderr from attach stream.

Runtime endpoint can be set with `GITY_RUNNER_CONTAINER_RUNTIME_ENDPOINT`.

## containerd

`containerd` uses the same API-driven flow as `docker`/`podman`.

Required values:

- `containerd`: `GITY_RUNNER_CONTAINER_RUNTIME_ENDPOINT`

## Firecracker (experimental)

`firecracker` currently supports runtime reachability checks and validation:

- validate socket config (`GITY_RUNNER_FIRECRACKER_SOCKET`);
- validate workspace path;
- probe the Firecracker API by making a Go-native HTTP request over unix socket/TCP.

Execution behavior:

- if the configured endpoint looks like a container runtime socket, the runner falls back to the existing
  container execution path for now (`container-runtime` compatibility mode) and marks the runtime as `firecracker`;
- otherwise, job execution returns an explicit "not implemented yet" error indicating full VM orchestration is pending.

Related env:

- `GITY_RUNNER_FIRECRACKER_SOCKET`
- `GITY_RUNNER_CONTAINER_IMAGE` (used for target job image metadata until execution is added)

## Security and Deployment Notes

- The runner is **trusted-host** by design in this beta release: it executes repository-provided job scripts and has access to the project token used for claiming jobs.
- Container runtimes provide a process boundary only for jobs that use container mode; host mode and the future Firecracker path do not provide the same degree of isolation yet.
- Keep container images for CI to the minimum privilege needed, and avoid mounting host paths other than the workspace bind.
- Masked CI variables are redacted in API responses and logs when possible, but mask coverage is a defense-in-depth aid, not a security boundary by itself.
- Keep workspace cleanup enabled (`GITY_RUNNER_CLEAN_WORKSPACE=true`) for untrusted scripts to reduce artifact persistence.

## Security Boundary Matrix

| Execution mode | Isolation boundary | Typical risk control |
| --- | --- | --- |
| `host` | none | do not assign untrusted projects to `host`; isolate with runner token scoping |
| `docker` | container namespace + cgroups + seccomp profile | restrict image allowlist, remove bind mounts, use read-only rootfs when possible |
| `podman` | container namespace + sandbox policy | prefer user namespace, drop extra capabilities |
| `containerd` | container namespace + runtime hooks | configure strict socket ACLs and workload user mapping |
| `firecracker` | VM boundary (experimental) | strongest tenant separation target; currently not feature-complete |

## Security Hardening Checklist

Before allowing untrusted workloads:

- Enforce runner tags by scope so projects can only claim jobs from intended runner groups.
- Use scoped runner tokens and short job lease windows.
- Keep `GITY_RUNNER_CLEAN_WORKSPACE=true`.
- Require workspace scratch space on node-local storage, not shared mounts.
- Enforce shell allowlist and deny unknown shells by default.
- Record critical runner lifecycle events (claim, trace, artifact upload, completion) into audit stream.

## Recommended runtime profile

| Environment | Suggested mode | Why |
| --- | --- | --- |
| Local dev or secure lab | `docker` or `podman` | Quick setup, easy local debugging |
| Shared/self-hosted runner host | `containerd` or `podman` | Better control and explicit socket binding |
| Public/untrusted jobs (future) | `firecracker` (when implemented) | VM-level isolation target for stronger tenant isolation |
