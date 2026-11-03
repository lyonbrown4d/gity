# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.99.1

# Reuse the signed upstream binary from the exact Angos release and commit
# approved for Gity; this stage does not copy or build the upstream source tree.
FROM ghcr.io/project-angos/angos@sha256:1b90f6720c43eb143f519aec4c7bb721f180e74536634b64912085e464c65abd AS angos-engine

FROM rust:${RUST_VERSION}-bookworm AS source
ARG RUST_VERSION
ARG TARGETARCH
ARG CARGO_BUILD_JOBS=4
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS} RUSTUP_TOOLCHAIN=${RUST_VERSION}
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev git ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY apps ./apps
COPY xtask ./xtask

FROM source AS contract
RUN --mount=type=cache,id=gity-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=gity-target-${TARGETARCH},target=/src/target,sharing=locked \
    mkdir -p dist/web /out && \
    cargo run --release --locked -p gity-server -- openapi --output /out/openapi.json

FROM node:24-bookworm AS web
RUN npm install --global pnpm@12.15.0
WORKDIR /src/web
COPY web/package.json web/pnpm-lock.yaml web/pnpm-workspace.yaml ./
RUN --mount=type=cache,id=gity-pnpm,target=/root/Library/pnpm/store,sharing=locked \
    pnpm install --frozen-lockfile
COPY web ./
COPY crates/gity-i18n/locales /src/crates/gity-i18n/locales
COPY --from=contract /out/openapi.json ./openapi.json
RUN pnpm gen:api && pnpm build

FROM source AS native
COPY --from=web /src/dist/web /src/dist/web
RUN --mount=type=cache,id=gity-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=gity-target-${TARGETARCH},target=/src/target,sharing=locked \
    touch dist/web && \
    cargo build --release --locked -p gity-server -p gity-runner -p gity-git-hook -p gity-search-index \
    && mkdir -p /opt/gity-bin \
    && cp target/release/gity-server target/release/gity-runner target/release/gity-git-hook target/release/gity-search-index /opt/gity-bin/

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates git openssh-client libssl3 libsqlite3-0 curl tzdata && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 gity && useradd --uid 10001 --gid gity --create-home gity \
    && mkdir -p /var/lib/gity && chown gity:gity /var/lib/gity
USER gity
WORKDIR /var/lib/gity

FROM docker:28.5-cli AS docker-client
FROM runtime AS runner
COPY --from=native /opt/gity-bin/gity-runner /usr/local/bin/gity-runner
COPY --from=docker-client /usr/local/bin/docker /usr/local/bin/docker
ENTRYPOINT ["gity-runner"]

FROM runtime AS search-index
COPY --from=native /opt/gity-bin/gity-search-index /usr/local/bin/gity-search-index
ENTRYPOINT ["gity-search-index"]

FROM runtime AS server
COPY --from=native /opt/gity-bin/gity-server /usr/local/bin/gity-server
COPY --from=native /opt/gity-bin/gity-git-hook /usr/local/bin/gity-git-hook
COPY --from=web /src/dist/web /opt/gity/web
ENV GITY_WEB__ROOT=/opt/gity/web GITY_GIT__HOOK=/usr/local/bin/gity-git-hook
EXPOSE 8080 2222
HEALTHCHECK --interval=10s --timeout=3s --retries=10 CMD curl --fail http://localhost:8080/api/health || exit 1
ENTRYPOINT ["gity-server"]
CMD ["standalone"]

# Consumers use this base only for the registry and full standalone targets.
# The regular server, API, git-node and runner images do not ship Angos.
FROM runtime AS runtime-with-angos
COPY --from=angos-engine --chmod=0555 /angos /usr/local/bin/angos
