<div lang="en">

[简体中文](../zh/container-stats.md) · [English](container-stats.md)

# Container stats sampling

`cargo run -p xtask -- container-stats --container <id-or-name> --duration-seconds 30 --label idle --output /tmp/container-stats.jsonl` records the Docker CLI's streaming `docker stats --format '{{json .}}'` samples as JSON Lines. Each record has `timestamp_unix`, `label`, and the unmodified Docker stats object (including CPU, memory, PIDs, and network fields). The output path is created exclusively; an existing file causes an error.

The sampler stops at the requested deadline, kills and waits for its own Docker CLI process, and never inspects container environment variables. Docker must be installed and the current user must have access to its daemon. This captures the Docker stats view, not process RSS; it does not infer or report RSS. Capture a separate `/proc/1/status` or cgroup snapshot when process or cgroup memory is required.

For a black-box run against an isolated short-lived container:

```sh
docker run -d --rm --name gity-stats-smoke alpine:3.22 sleep 90
cargo run -p xtask -- container-stats --container gity-stats-smoke --duration-seconds 5 --label smoke --output /tmp/gity-stats-smoke.jsonl
docker stop gity-stats-smoke
```

Inspect the JSONL records and verify the Docker CLI has exited after the five-second window. Use a fresh output path for each run.

</div>
