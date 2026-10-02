ARG SPACK_VERSION=v2.1.5

FROM node:22-alpine AS build

WORKDIR /app

RUN corepack enable && corepack prepare pnpm@10 --activate

COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

COPY . .
RUN pnpm build:ci

FROM ghcr.io/lyonbrown4d/spack-compiler:${SPACK_VERSION} AS spack-compile

WORKDIR /workspace

COPY --from=build /app/dist /workspace/dist

RUN spack-compiler compile /workspace/dist \
    --output /tmp/app.spack \
    --assets.path=/ \
    --assets.entry=index.html \
    --assets.fallback.on=not_found \
    --assets.fallback.target=index.html \
    --compression.enable=true \
    --compression.mode=warmup \
    --compression.cache_dir=/tmp/spack-cache \
    --image.enable=false \
    --frontend.resource_hints.enable=false

FROM ghcr.io/lyonbrown4d/spack:${SPACK_VERSION}

COPY --from=spack-compile /tmp/app.spack /app/app.spack

ENV SPACK_ASSETS_ROOT=/app/app.spack \
    SPACK_ASSETS_PATH=/ \
    SPACK_ASSETS_ENTRY=index.html \
    SPACK_ASSETS_FALLBACK_ON=not_found \
    SPACK_ASSETS_FALLBACK_TARGET=index.html \
    SPACK_HTTP_PORT=8080 \
    SPACK_COMPRESSION_ENABLE=true \
    SPACK_COMPRESSION_MODE=off \
    SPACK_IMAGE_ENABLE=false \
    SPACK_LOGGER_LEVEL=info

CMD ["--frontend.resource_hints.enable=false"]

EXPOSE 8080
