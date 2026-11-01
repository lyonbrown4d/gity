<div lang="en">

[简体中文](../zh/performance-git-client.md) · [English](performance-git-client.md)

# Git Client Stress Tests

The k6 workload covers REST APIs. Git clone/fetch/push needs a real Git client because Smart HTTP uses Git protocol RPCs over `info/refs`, `git-upload-pack`, and `git-receive-pack`.

The `xtask git-stress` command seeds a project through the API, creates a project access token with `read_repository` and `write_repository`, then runs concurrent real `git` subprocesses.

## Run Against The Blackbox Stack

```bash
cargo run -p xtask -- git-stress --build --workers 4 --iterations 24
```

The command starts `docker-compose.blackbox.yaml` and keeps the server stack running. Local temporary Git worktrees are cleaned by default. For a project created by the command, it revokes the access token and marks the project for deletion after the run. The generated account and organization remain in the test database for database-lifecycle cleanup.

## Run Against An Existing Server

```bash
cargo run -p xtask -- git-stress --no-compose --base-url http://127.0.0.1:18080 --workers 8 --iterations 80
```

## Workload Modes

```bash
cargo run -p xtask -- git-stress --mode mixed --workers 8 --iterations 100
cargo run -p xtask -- git-stress --mode clone --workers 4 --iterations 20
cargo run -p xtask -- git-stress --mode fetch --workers 16 --iterations 200
cargo run -p xtask -- git-stress --mode push --workers 4 --iterations 40
cargo run -p xtask -- git-stress --mode ls-remote --workers 32 --iterations 500
```

Mixed mode defaults to `clone=20`, `fetch=45`, `push=25`, and `ls-remote=10`. Override with `--clone-percent`, `--fetch-percent`, `--push-percent`, and `--ls-remote-percent`.

## Existing Repository Mode

If you already have a repository URL and token:

```bash
GITY_GIT_PERF_TOKEN=<token> cargo run -p xtask -- git-stress --no-compose --project-url http://127.0.0.1:18080/group/project.git --workers 8 --iterations 80
```

The token is read from `GITY_GIT_PERF_TOKEN` and supplied to Git through a temporary askpass helper. Override its username with `--token-username`. The token is not placed in the repository URL or Git arguments. Existing-repository mode does not delete the supplied project or its remote branches. The askpass helper requires a Unix shell; Windows Git stress execution has not been verified.

## Proxy And Credential Helpers

For localhost targets, the runner bypasses Git/HTTP proxies and clears Git credential helpers for subprocesses. This keeps blackbox runs independent from machine-level proxy settings such as `HTTP_PROXY` or Git credential manager configuration.

The server must return `WWW-Authenticate: Basic` on Git Smart HTTP authentication failures. Git clients use that challenge before retrying URL credentials for private repositories.

## Server-Side Git RPC Capacity

The server limits concurrent native Git Smart HTTP subprocesses with `git.max_concurrent_rpc` (`GITY_GIT__MAX_CONCURRENT_RPC`). The default is `32`; set it to `0` to disable the limiter. Raise it only when the host has enough CPU, file descriptors, and repository I/O capacity. Lower it if mixed API + Git traffic starts producing gateway errors or long tail latency.

## Metrics

The runner prints per-operation count, failures, avg, p50, p95, and max for:

- `setup.init`, `setup.commit`, `setup.push`, `setup.clone`.
- `ls-remote` for ref advertisement/read checks.
- `clone` for fresh client clone flows.
- `fetch` for incremental `git-upload-pack` flows.
- `push` for `git-receive-pack` flows.

`--keep-workdir` preserves local worktrees for debugging. When the command creates its own project, project deletion is requested and its test token is revoked regardless of this flag. Branches pushed to an existing external project are left in place.

</div>
