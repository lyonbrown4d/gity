<div lang="en">

[简体中文](../zh/ci-yaml.md) · [English](ci-yaml.md)

# CI YAML

The Rust server reads the repository's `.gity-ci.yml` by default. The API can also accept inline YAML validated by the same rules. Content must be UTF-8 and no larger than 1 MiB. Jobs must be under an explicit `jobs` mapping; configuration uses typed YAML. Inline configuration cannot declare itself to be trusted repository configuration.

```yaml
name: build-and-test
stages: [build, test]
image: alpine:3.22
variables:
  BUILD_MODE: release
jobs:
  compile:
    stage: build
    script:
      - mkdir -p output
      - printf '%s\n' "$BUILD_MODE" > output/mode.txt
    artifacts:
      paths: [output/mode.txt]
    timeout_seconds: 300
  verify:
    stage: test
    needs: [compile]
    script: test "$BUILD_MODE" = release
    timeout_seconds: 60
```

`script` accepts a string or an array of strings. Project, global, and job-level variables are merged by scope; the Runner injects `CI_PROJECT_PATH`、`CI_COMMIT_SHA`、`CI_COMMIT_REF_NAME`、`CI_JOB_NAME`, and other runtime context.

If `stages` is omitted, the default stages are `build`, `test`, and `deploy`; a job defaults to the `test` stage. Omitting `needs` means depending on all jobs in preceding stages `needs: []` means there are no prior job dependencies. Explicit dependencies must exist and be in the current or an earlier stage; cycles are not allowed.

A configuration supports 1 to 200 jobs, with timeouts from 1 to 86400 seconds. Use `tags` to match a Runner. Unknown fields are rejected so that typos are not silently ignored.

Artifact paths must be relative to the working directory and cannot contain traversal or wildcards. Artifacts are retrieved separately through the download API. Dependencies currently control scheduling order; artifacts from prior jobs are not automatically extracted into later jobs.

## Runner

After registering a project Runner, pass its project-specific token to the standalone process:

```sh
GITY_RUNNER__TOKEN='project-runner-token' cargo run -p gity-runner -- --server http://localhost:8080
```

Container execution is the default and requires the Docker CLI and an available Docker daemon.`--executor shell` runs scripts on the host and is only for trusted local jobs.`--once` exits after one claim cycle. See the full options in `gity-runner --help`。

The Runner streams a source ZIP for a fixed commit, sends job heartbeats, chunked logs, and artifacts, then submits the completion status. The job lease limits the Runner's write permissions; lease expiration, cancellation, and timeout terminate the execution process.

Project variables marked `protected` are provided only to the default branch or branches matching protection rules. The commit must still be the branch's current tip, and the YAML must match the content at the repository’s pinned commit. A client-supplied `config_source` does not make configuration trusted. Values marked `masked` are replaced before logs are persisted; masking must also work across log chunks.

## API

For project pipelines, YAML validation, job retries, cancellation, Runner registration, job claims, and artifact endpoints, refer to `/docs` on the running service. Endpoint definitions are collected from the actual Axum routes and Utoipa macros; download `/openapi.json` for contract checks.

</div>
