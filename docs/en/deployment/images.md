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

The Dockerfile separates system build dependencies, Rust source, and Web tools into reusable stages. Debian uses BuildKit to cache apt indexes and packages; Alpine caches APK packages; Cargo registry, Git checkouts, and target artifacts are cached separately for each base system; the target cache uses an exclusive lock during builds and artifact copying, preventing parallel roles or architectures from overwriting files being packaged. `wasm-bindgen` and `wasm-opt` tools are cached by base system and architecture. `GITY_ROLE` and `CARGO_JOBS` affect only native Cargo build steps. Cargo runs separately for each role, so one role's feature set is not included in another role's Cargo build invocation. Trunk and `wasm32-unknown-unknown` are installed before Rust source is copied; the Web build does not depend on the native role builder.

Dockerfile supports Debian and Alpine role combinations, but the presence of configuration or a successful `cargo build` does not prove that the resulting container runs. The full image black-box matrix is incomplete. See the [Validation Record](../verification.md).
## Image Black-Box Acceptance

Run from the repository root, using a separate, unused Compose project name each time:

```sh
cargo xtask blackbox --mode standalone --base debian \
  --project gity-bb-acceptance-standalone-debian \
  --evidence /tmp/gity-standalone-debian.json
```

Run all four combinations of `--mode standalone|distributed` and `--base debian|alpine`. The tool builds the actual images and starts isolated data volumes. It checks native package clients, Git HTTP/SSH, the CI Runner, OCI/mirror, and permission isolation. Distributed runs also check component failures and controlled Git promotion. By default, it removes the project and its data volumes when finished, and refuses to take over an existing project. The host needs available HTTP 8080, SSH 2222, and OCI 5000 ports, plus Docker, Git, SSH, and OpenSSL.

If downloads need a proxy, set `GITY_BLACKBOX_DOWNLOAD_PROXY=http://proxy-reachable-from-container:port`. The tool accepts only an HTTP proxy URL without credentials and passes it to the Compose services in this run and the temporary package client; internal services connect directly. Evidence records image IDs, actual operating systems, UIDs, and one container resource snapshot. On failure, it also saves redacted logs. A snapshot is not a stress test.

</div>
