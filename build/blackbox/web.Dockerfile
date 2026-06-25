FROM node:22-alpine AS build

WORKDIR /app

RUN corepack enable && corepack prepare pnpm@10 --activate

COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

COPY . .
RUN pnpm build:ci

FROM ghcr.io/lyonbrown4d/spack:latest

COPY --from=build /app/dist /app

ENV SPACK_ASSETS_ROOT=/app
ENV SPACK_ASSETS_PATH=/
ENV SPACK_ASSETS_ENTRY=index.html
ENV SPACK_ASSETS_FALLBACK_TARGET=index.html
ENV SPACK_HTTP_PORT=8080

EXPOSE 8080
