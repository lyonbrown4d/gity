# k6 Performance Tests

Gity keeps k6 API performance tests separate from the Docker health/OpenAPI/Web smoke and real Git client stress checks. The main workload seeds one authenticated organization/project once, then exercises common API paths with weighted traffic.

## Run Against The Blackbox Stack

```bash
cargo run -p xtask -- k6 --build --vus 8 --duration 30s
```

If local `k6` is not installed, the runner automatically falls back to `grafana/k6` through Docker. On Linux it uses host networking so the container can reach the blackbox port bound to loopback. On other platforms it uses Docker Desktop's `host.docker.internal` route; use `--docker-base-url` to provide a reachable URL explicitly. You can force Docker k6 explicitly:

```bash
cargo run -p xtask -- k6 --build --docker-k6 --vus 8 --duration 30s
```

The runner starts `docker-compose.blackbox.yaml` and keeps the stack running after the test so the generated data can be inspected.

## Run Against An Existing Server

```bash
GITY_K6_BASE_URL=http://127.0.0.1:18080 GITY_K6_VUS=16 GITY_K6_DURATION=1m k6 run perf/k6/common-api.js
```

Or use the Rust xtask runner without touching Docker Compose:

```bash
cargo run -p xtask -- k6 --no-compose --base-url http://127.0.0.1:18080 --vus 16 --duration 1m
```

## Tunables

- `GITY_K6_BASE_URL`: web gateway URL, default `http://127.0.0.1:18080`.
- `GITY_K6_VUS`: constant VU count, default `8`.
- `GITY_K6_DURATION`: scenario duration, default `30s`.
- `GITY_K6_P95_MS`: p95 latency threshold in milliseconds, default `1000`.
- `GITY_K6_WRITE_PERCENT`: percentage of iterations assigned to light write flows, default `12`.
- `GITY_K6_DEBUG_FAILURES=true`: print failing response bodies.

The runner exposes `--base-url`, `--vus`, `--duration`, `--p95`, and `--write-percent` as typed CLI options. It uses local `k6` when available and otherwise runs `grafana/k6` through Docker.

## Covered API Areas

- Auth, users, organizations, projects, members, and permissions.
- Repository branches, branch protections, commits, tree, blob, README, search, and language stats.
- Issues, comments, assignees, labels, merge requests, approvals, participants, checks, and diffs.
- Wiki pages and package registry JSON/generic/PyPI/NuGet read paths.
- Pipelines, jobs, runner registration/heartbeat/claim, and CI variables.
- LFS management and audit events.

The script tags dynamic routes with stable k6 `name` values so metrics stay low-cardinality.
