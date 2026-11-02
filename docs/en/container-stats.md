<div lang="en">

[简体中文](../zh/container-stats.md) · [English](container-stats.md)

# Container stats sampling

`cargo run -p xtask -- container-stats --container <id-or-name> --duration-seconds 30 --label idle --output /tmp/container-stats.jsonl` records the Docker CLI's streaming `docker stats --format '{{json .}}'` samples as JSON Lines. Each record has `timestamp_unix`, `frame`, `label`, and the unmodified Docker stats object (including CPU, memory, PIDs, and network fields). The output path is created exclusively; an existing file causes an error.

The sampler stops at the requested deadline, kills and waits for its own Docker CLI process, and never inspects container environment variables. Docker must be installed and the current user must have access to its daemon. This captures the Docker stats view, not process RSS; it does not infer or report RSS. Capture a separate `/proc/1/status` or cgroup snapshot when process or cgroup memory is required.

For a black-box run against an isolated short-lived container:

```sh
docker run -d --rm --name gity-stats-smoke alpine:3.22 sleep 90
cargo run -p xtask -- container-stats --container gity-stats-smoke --duration-seconds 5 --label smoke --output /tmp/gity-stats-smoke.jsonl
docker stop gity-stats-smoke
```

Inspect the JSONL records and verify the Docker CLI has exited after the five-second window. Use a fresh output path for each run.

One `--container` option can list multiple IDs or names, for example `--container gity-api gity-git-node gity-git-node-2`. The sampler shares one Docker stats stream; `frame` groups distinct container records in one cycle and advances when an ID repeats. Aggregate memory only for complete frames containing every target; summing component peaks from different times is not a simultaneous peak. Docker CPU `100%` represents approximately one logical CPU. Memory retains the Docker stats meaning.

For a stream spanning several load stages, use `--stop-file /tmp/gity-stats.stop --duration-seconds 3600`. The file must initially be absent. Create it after the load stages finish; the sampler stops within approximately one second, reaps its Docker CLI and completes the output. Filter resource records by each benchmark JSON's measurement timestamps to separate setup, warm-up and measured work.

</div>
