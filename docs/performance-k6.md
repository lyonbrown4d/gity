# k6 Performance Tests

Gity keeps k6 performance tests separate from Playwright E2E and blackbox UI tests. The main workload seeds one authenticated organization/project once, then exercises common API paths with weighted traffic.

## Run Against The Blackbox Stack

```bash
node scripts/k6-blackbox.mjs --build --vus 8 --duration 30s
```

If local `k6` is not installed, the runner automatically falls back to `grafana/k6` through Docker. You can force Docker k6 explicitly:

```bash
node scripts/k6-blackbox.mjs --build --docker-k6 --vus 8 --duration 30s
```

The runner starts `docker-compose.blackbox.yaml` and keeps the stack running after the test so the generated data can be inspected.

## Run Against An Existing Server

```bash
GITY_K6_BASE_URL=http://127.0.0.1:18080 GITY_K6_VUS=16 GITY_K6_DURATION=1m k6 run perf/k6/common-api.js
```

Or use the cross-platform Node runner without touching Docker Compose:

```bash
node scripts/k6-blackbox.mjs --no-compose --base-url http://127.0.0.1:18080 --vus 16 --duration 1m
```

## Tunables

- `GITY_K6_BASE_URL`: web gateway URL, default `http://127.0.0.1:18080`.
- `GITY_K6_VUS`: constant VU count, default `8`.
- `GITY_K6_DURATION`: scenario duration, default `30s`.
- `GITY_K6_P95_MS`: p95 latency threshold in milliseconds, default `1000`.
- `GITY_K6_WRITE_PERCENT`: percentage of iterations assigned to light write flows, default `12`.
- `GITY_K6_DEBUG_FAILURES=true`: print failing response bodies.

The Node runner exposes the same options as CLI flags: `--base-url`, `--vus`, `--duration`, `--p95`, and `--write-percent`.

## Covered API Areas

- Auth, users, organizations, projects, members, and permissions.
- Repository branches, branch protections, commits, tree, blob, README, search, and language stats.
- Issues, comments, assignees, labels, merge requests, approvals, participants, checks, and diffs.
- Wiki pages and package registry JSON/generic/PyPI/NuGet read paths.
- Pipelines, jobs, runner registration/heartbeat/claim, and CI variables.
- LFS management and audit events.

The script tags dynamic routes with stable k6 `name` values so metrics stay low-cardinality.
