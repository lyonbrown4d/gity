<div lang="en">

[简体中文](../../zh/deployment/images.md) · [English](images.md)

# Container Image Variants

[`Dockerfile.variants`](https://github.com/lyonbrown4d/gity/blob/main/build/package/Dockerfile.variants) builds a separate image for each application role. By default it uses `rust:1.99.1-bookworm` to compile and`debian:bookworm-slim` to run. The Alpine combination uses `rust:1.99.1-alpine` and `alpine:3.22` to produce musl binaries. Pair the Debian builder with the Debian runtime, and the Alpine builder with the Alpine runtime.

| `GITY_ROLE` / target | Image contents |
| --- | --- |
| `standalone` | Full standalone application, static Web assets, Git hooks, Runner and Docker CLI, and the Angos OCI engine. |
| `api` | API application and static Web assets. |
| `git` | Git collaboration HTTP/SSH services, static Web assets, and Git hooks. |
| `git-node` | Git storage node and Git hooks. |
| `gateway` | HTTP/SSH ingress gateway; Web requests are served by the API service. |
| `registry` | OCI image gateway, package protocol service, and Angos OCI engine. |
| `runner` | Standalone Runner and Docker CLI. |
| `worker` | Background Worker. |

All Gity runtime containers run as UID/GID `10001`. Application roles use the shared `gity-observability` library; each runtime image contains only the programs and static assets required for that role. Angos uses an upstream binary pinned by digest; this Dockerfile does not build Angos from source.

## Build Locally

From the repository root, use BuildKit to build a Debian standalone image for the host architecture:

```sh
docker buildx build --load \
  -f build/package/Dockerfile.variants \
  --build-arg GITY_ROLE=standalone \
  --build-arg GITY_BASE=debian \
  --build-arg BUILDER_IMAGE=rust:1.99.1-bookworm \
  --build-arg RUNTIME_IMAGE=debian:bookworm-slim \
  --target standalone \
  --tag gity:local-standalone-debian \
  .
```

To build an Alpine Registry image, change the role, base system, and tag to the corresponding values:

```sh
docker buildx build --load \
  -f build/package/Dockerfile.variants \
  --build-arg GITY_ROLE=registry \
  --build-arg GITY_BASE=alpine \
  --build-arg BUILDER_IMAGE=rust:1.99.1-alpine \
  --build-arg RUNTIME_IMAGE=alpine:3.22 \
  --target registry \
  --tag gity:local-registry-alpine \
  .
```

`GITY_ROLE` must match `--target`. Optional parameter `CARGO_JOBS` defaults to `4`. Compose already sets the target and arguments for each role. To deploy with Compose, follow the [Component Deployment Guide](compose.md).

## Build Cache and Verification Scope

The Dockerfile caches system dependencies, Cargo dependencies, and the global pnpm store separately. A contract-export tool that is never shipped generates OpenAPI; Orval then generates the client and Vite builds the current React frontend. Each role finally compiles its Rust binary with `dist/web` embedded. Trunk and Rust frontend tooling are not used. Cargo target caches are isolated by base system and exclusively locked during compilation and artifact copying; `CARGO_JOBS` bounds Rust compilation concurrency.

Dockerfile supports Debian and Alpine role combinations, but the presence of configuration or a successful `cargo build` does not prove that the resulting container runs. Run each image and architecture combination independently. See the [Validation Record](../verification.md).
## Image Black-Box Acceptance

Run from the repository root, using a separate, unused Compose project name each time:

```sh
cargo xtask blackbox --mode standalone --base debian \
  --project gity-bb-acceptance-standalone-debian \
  --evidence /tmp/gity-standalone-debian.json
```

Run all four combinations of `--mode standalone|distributed` and `--base debian|alpine`. The tool builds the actual images and starts isolated data volumes. It checks native package clients, Git HTTP/SSH, the CI Runner, OCI/mirror, and permission isolation. Distributed runs also check component failures and controlled Git promotion. By default, it removes the project and its data volumes when finished, and refuses to take over an existing project. The host needs available HTTP 8080, SSH 2222, and OCI 5000 ports, plus Docker, Git, SSH, and OpenSSL.

Add `--suite protocols` to check the current Web assets, API, Git HTTP/SSH, MR recovery, OCI/mirror, and distributed failures. This scope explicitly skips the CI Runner, native language package clients, and backup/restore. It is not equivalent to the default `--suite full` acceptance; the evidence records the selected `suite`.

If downloads need a proxy, set `GITY_BLACKBOX_DOWNLOAD_PROXY=http://proxy-reachable-from-container:port`. The tool accepts only an HTTP proxy URL without credentials and passes it to the Compose services in this run and the temporary package client; internal services connect directly. Evidence records image IDs, actual operating systems, UIDs, and one container resource snapshot. On failure, it also saves redacted logs. A snapshot is not a stress test.

Use `--no-build` only after building current-source image tags for the same `--project`, role, and base system. It still runs every functional check and records image IDs; old images cannot validate new source. Cargo downloads disable HTTP/2 multiplexing to reduce proxy stream resets. Restricted networks can pass standard Docker `HTTP_PROXY` and `HTTPS_PROXY` build arguments during prebuilds.

The download proxy must be reachable from both Compose containers and the DIND namespace. Avoid `127.0.0.1` and address conflicts with DIND's default `172.17.0.0/16` network; a dedicated test bridge can forward a local proxy. SQLite fault injection uses the official `python:3.13.2-bookworm` image as a database client; pull it before acceptance. Metadata-preserving backups use the GNU tar helper in `debian:bookworm-slim`. Both are test/operations tools and are excluded from application images.

</div>
