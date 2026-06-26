FROM node:22-alpine AS build

WORKDIR /app

RUN corepack enable && corepack prepare pnpm@10 --activate

COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

COPY . .
RUN pnpm build:ci

FROM ghcr.io/lyonbrown4d/spack-compiler:latest AS spack-compile

WORKDIR /workspace

COPY --from=build /app/dist /workspace/dist

RUN /opt/spack-compiler \
    --assets.path=/ \
    --assets.entry=index.html \
    --assets.fallback.on=not_found \
    --assets.fallback.target=index.html \
    --compression.enable=true \
    --compression.mode=warmup \
    --compression.cache_dir=/tmp/spack-cache \
    --image.enable=false \
    --frontend.resource_hints.enable=false \
    compile /workspace/dist -o /workspace/app.spack

FROM ghcr.io/lyonbrown4d/spack:latest

COPY --from=spack-compile /workspace/app.spack /app/app.spack

CMD ["--assets.root=/app/app.spack", "--assets.path=/", "--assets.entry=index.html", "--assets.fallback.on=not_found", "--assets.fallback.target=index.html", "--http.port=8080", "--compression.enable=true", "--compression.mode=off", "--image.enable=false", "--frontend.resource_hints.enable=false", "--logger.level=info"]

EXPOSE 8080