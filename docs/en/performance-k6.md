<div lang="en">

[简体中文](../zh/performance-k6.md) · [English](performance-k6.md)

# k6 Performance Tests

Use `perf/k6/git-api.js` for Git REST load. It is separate from real Git/SSH client stress, functional black-box acceptance, and the general `common-api.js` workload. It writes data; use a dedicated acceptance environment and start sampling after builds and functional checks finish.

## Git REST Read/Write Workload

Setup registers isolated owner and outsider accounts, each with a personal namespace, and creates a private project under the owner namespace. Passwords use k6 `crypto.randomBytes`. The repository contains a README and approximately 1 MiB of random ASCII content. It has no `.gity-ci.yml`, so Git load does not enqueue Runner jobs.

By default, approximately 20% of iterations commit a file. Other iterations choose evenly among branches, commit history, tree, and blob. Each VU adds one outsider authorization probe every 100 iterations:

| Endpoint | Validation |
| --- | --- |
| `GET /projects/{id}/repository/branches` | Main retains the seed SHA; every branch returns a valid commit SHA. |
| `GET /projects/{id}/repository/commits` | Main history contains the seed commit with valid SHAs. |
| `GET /projects/{id}/repository/tree` | Seed file path, kind, and size match. |
| `GET /projects/{id}/repository/blob` | Full 1 MiB size and SHA-256 match. |
| `POST /projects/{id}/file-commits` | Each VU owns a branch and creates a unique file per iteration; validate the returned SHA, then read by that immutable SHA and compare the content. |
| Outsider blob read | Private project returns expected `403`. |

Write percentage describes iterations, not HTTP request count: each successful write adds a verification read. Teardown only marks this run's project deleted and logs out its two sessions. It does not clean other projects, accounts, or namespaces. The acceptance operator removes the entire temporary stack afterward. A partial setup failure also attempts to clean this run's created project and sessions.

## Run Against an Existing Server

Create a fresh output directory first. This example runs 1, 8, and 32 VUs sequentially for 60 seconds each, preserving separate JSON reports:

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

`--no-compose` neither starts nor cleans the server. Set the actual endpoint URL. The example's `--p95 5000` is a run threshold, not a performance guarantee. Record deployment layout, base image, database, machine resources, and revision. Avoid concurrent builds or unrelated workloads while sampling. Results from different machines or dataset sizes are not interchangeable baselines. Blob checks parse and hash 1 MiB on the load generator; monitor k6 CPU too, so a client bottleneck is not mistaken for server capacity.

Without `--docker-k6`, an installed local `k6` is preferred. Otherwise the runner uses the pinned official image `grafana/k6:1.8.2`, never `latest`; see the [official release](https://github.com/grafana/k6/releases/tag/v1.8.2). Linux defaults to host networking. Other platforms use Docker Desktop's `host.docker.internal`; `--docker-base-url` provides an explicit container-reachable URL.

`--summary-export` requires an existing parent directory and a new destination file. The runner atomically reserves the destination, lets k6 write JSON into an isolated private temporary directory, validates it, and publishes it into the reserved file. Docker receives only that temporary output directory as a writable mount and uses the caller's UID/GID. Existing reports are not overwritten. Valid JSON is retained after a threshold failure; an empty reservation is removed if startup produced no result.

## Metrics and Expected Errors

Every stable endpoint name has a `git_api_<endpoint>_duration` Trend, `git_api_<endpoint>_unexpected_failure` Rate, and `requests` / `successes` Counters. Trend summaries include `med` (p50), p95, p99, min, and max. Dynamic project IDs, VU branches, and tokens are not metric labels. Native k6 `--summary-export` JSON preserves these metrics and threshold results.

`git_api_expected_forbidden` counts authorization probes separately. `git_api_expected_conflicts` counts `409` only when explicitly allowed in shared-branch mode. The new script uses the k6 [expectedStatuses](https://grafana.com/docs/k6/latest/javascript-api/k6-http/expected-statuses/) callback for expected statuses and dedicated Rates/content checks for unexpected failures. It does not change the shared helper used by old workloads. An unexpected `403` or `409` on a normal read/write still fails and is not successful throughput.

Defaults require all contract checks to pass, zero unexpected failures, and at least one successful commit with exact readback when writes are enabled. Every endpoint's p95 must be below the configured threshold. Optional `GITY_K6_CONFLICTS=true` makes VUs share `k6-shared` to inspect contention separately. This mode still requires real successful writes; it cannot pass by measuring only `409` responses.

## Parameters and Other Workloads

| Environment variable | Git script default | Rust runner option |
| --- | --- | --- |
| `GITY_K6_BASE_URL` | `http://127.0.0.1:18080` | `--base-url` |
| `GITY_K6_VUS` | `8` | `--vus` |
| `GITY_K6_DURATION` | `60s` | `--duration` (runner defaults to `30s`; set explicitly) |
| `GITY_K6_P95_MS` | `1000` | `--p95` |
| `GITY_K6_WRITE_PERCENT` | `20` | `--write-percent` |
| `GITY_K6_CONFLICTS` | `false` | Inherited calling environment; local and Docker supported. |

Direct `k6 run perf/k6/git-api.js` reads these variables and the existing `K6_*` aliases. The Rust runner passes typed CLI parameters into the script. Its default write percentage is `20` for the Git script and remains `12` for `common-api.js`. The script never prints passwords, tokens, or complete session responses.

Select the general API workload with `--script perf/k6/common-api.js`. Omitting `--no-compose` starts `docker-compose.blackbox.yaml` and retains it after testing. `--build` builds during startup and should not be used during formal sampling.

</div>
