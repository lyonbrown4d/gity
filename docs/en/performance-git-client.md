<div lang="en">

[简体中文](../zh/performance-git-client.md) · [English](performance-git-client.md)

# Git Client Stress Tests

`cargo xtask git-stress` runs real Git clone, fetch, push and ls-remote clients over HTTP or OpenSSH. By default it creates an isolated private project through the API. HTTP uses a project access token; SSH registers the supplied public key as a project deployment key with push access. Cleanup revokes that credential, requests project deletion and logs out. Generated accounts and organizations remain until test-database cleanup. Tokens and private keys are excluded from repository URLs, diagnostic logs and JSON summaries.

The runner requires Git, OpenSSH and a Unix shell. Binary fixtures read the operating system entropy source `/dev/urandom`. Windows execution has not been verified. k6 Git REST/API workloads and these native Git protocol workloads are measured separately.

## Connect To An Existing Server

HTTP is the default transport. This creates and tests an isolated project:

```bash
cargo xtask git-stress --no-compose \
  --base-url http://127.0.0.1:18080 \
  --workers 8 --iterations 80
```

SSH requires a dedicated noninteractive private key and a verified host public key file. Obtain the host public key from the environment operator before preparing known_hosts; the runner never accepts an unknown host automatically. The default SSH port is `2222`; the hostname comes from `--base-url`. Override them with `--ssh-port` and `--ssh-host`.

```bash
ssh-keygen -q -t ed25519 -N '' -f /tmp/gity-perf-key
# Example content: [127.0.0.1]:2222 ssh-ed25519 <verified-server-host-public-key>
chmod 600 /tmp/gity-perf-key /tmp/gity-perf-known_hosts
cargo xtask git-stress --no-compose --transport ssh \
  --base-url http://127.0.0.1:18080 --ssh-port 2222 \
  --ssh-key /tmp/gity-perf-key --ssh-known-hosts /tmp/gity-perf-known_hosts \
  --workers 8 --iterations 80
```

SSH uses the fixed `git` username, strict host verification and the supplied identity file. Machine SSH configuration and global known_hosts are ignored. Paths are passed through separately quoted arguments in a fixed wrapper, rather than assembled into executable shell text.

## Fixed Windows, Dataset Size And Clone Depth

Without `--duration-seconds`, `--iterations` is the total operation count across all workers. With it, each worker repeatedly performs an operation before requesting another within a shared fixed window; this is a closed loop, without a target arrival rate. At the deadline the runner stops starting operations and waits for in-flight operations. Actual measurement time can therefore exceed the requested window. `--timeout-ms` limits each Git subprocess and defaults to `60000`.

This configuration rewrites one 1 MiB random file in each of 16 commits: approximately a 1 MiB working tree and 16 MiB of random full history, plus a small README and Git metadata.

```bash
cargo xtask git-stress --no-compose --transport ssh \
  --base-url http://127.0.0.1:18080 \
  --ssh-key /tmp/gity-perf-key --ssh-known-hosts /tmp/gity-perf-known_hosts \
  --workers 8 --duration-seconds 60 --mode mixed \
  --seed-files 1 --bytes-per-file 1048576 --seed-commits 16 \
  --clone-depth 0 --summary target/git-stress-ssh.json
```

`--seed-files` defaults to `8`. Omitting `--bytes-per-file` retains small text fixtures, and `--seed-commits` defaults to `1`. A configured byte count generates fresh OS entropy for every file in every commit, preventing repeated filler from compressing heavily. For an 8 MiB working tree with four commits use `--seed-files 1 --bytes-per-file 8388608 --seed-commits 4`.

Measured clones default to `--clone-depth 1`. Zero means a full clone; positive integers specify shallow depth. Initial worker worktrees always fully clone main. API provisioning, fixture writes, the initial push and initial worker clones all belong to setup and are excluded from the measurement window.

## Operation Modes

`--mode` supports `mixed`, `clone`, `fetch`, `push` and `ls-remote`. Mixed defaults to clone 20, fetch 45, push 25 and ls-remote 10; use the corresponding `--*-percent` options to change the weights.

Each clone uses a new directory. Fetch updates an existing worker worktree; running it alone usually measures ref negotiation against an already synchronized repository. Push writes a small commit on a distinct `perf/<run>/<worker>-<iteration>` branch, avoiding contention on main. Mixed fetches can receive branches created by other workers.

## JSON And Latency Scope

`--summary PATH` writes JSON. The console also prints per-operation count, fail, avg, p50, p95, p99 and max, with latency in milliseconds. Percentiles use nearest rank and include failed and timed-out samples.

The `measurement.started_unix_seconds` and `finished_unix_seconds` fields are UTC Unix-second anchors recorded just before workers start together and after the last in-flight operation finishes, for filtering resource samples. Elapsed time and throughput continue to use a monotonic clock, so wall-clock adjustments do not change those measurements. The summary separates `setup` and `measurement`, each with actual `elapsed_seconds`, operation metrics and throughput. It records transport, workers, clone_depth, seed_files, bytes_per_file and seed_commits. `throughput_ops_per_second` divides all completed operations by actual measurement time; `successful_ops_per_second` counts only successful operations. `seed_applied` identifies whether this run generated the seed; configured seed values do not describe the actual contents of an existing repository. Workload failures still produce a summary and cause a nonzero exit status.

These are client end-to-end measurements, including process startup, local file operations and diagnostic collection. Push also includes checkout, add and commit; by default clone includes temporary-directory cleanup. Native child-completion waiting avoids fixed polling intervals, and timeout cancels and reaps the whole process group. Results are not server RPC latency. Setup warms repository, connection paths and file caches, so results do not represent cold starts either.

## Existing Repository Mode

HTTP reads `GITY_GIT_PERF_TOKEN` and supplies it through a temporary askpass helper. Override the variable name or username with `--token-env` and `--token-username`:

```bash
GITY_GIT_PERF_TOKEN=<token> cargo xtask git-stress --no-compose \
  --project-url http://127.0.0.1:18080/group/project.git --mode fetch --workers 8 --iterations 80
```

SSH uses a key already authorized for the repository. It does not read an HTTP token or call the API to create or delete a project:

```bash
cargo xtask git-stress --no-compose --transport ssh \
  --project-url ssh://git@127.0.0.1:2222/group/project.git \
  --ssh-key /tmp/gity-perf-key --ssh-known-hosts /tmp/gity-perf-known_hosts \
  --mode ls-remote --workers 8 --duration-seconds 60 --summary target/git-existing.json
```

The caller supplies permissions and prepared data for an existing repository; remote branches created by the command remain. `--keep-workdir` preserves local worktrees; default cleanup removes only the generated run directory. Localhost HTTP requests bypass machine proxies and Git credential helpers.

## Start Test Compose

Omitting `--no-compose` retains `docker-compose.blackbox.yaml` startup, with optional `--build`; the stack remains running afterward. SSH adds a temporary Compose override to enable and publish the selected SSH port. Operators must still verify and prepare known_hosts in advance. For production-image acceptance and performance sampling, connect to an already started isolated environment so builds, other tests and shared-service load do not contaminate measurements.

</div>
