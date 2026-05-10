# Production Deployment

The beta release publishes separate Docker images for each runtime:

- `ghcr.io/daiyuang/gity-server`
- `ghcr.io/daiyuang/gity-migration`
- `ghcr.io/daiyuang/gity-worker`
- `ghcr.io/daiyuang/gity-standalone`
- `ghcr.io/daiyuang/gity-runner`

## Split Process Mode

Copy the production example and update secrets, external URL, and image tag:

```bash
cp .env.production.example .env.production
```

Run migrations explicitly before server and worker:

```bash
docker compose --env-file .env.production -f docker-compose.prod.yaml --profile migration run --rm migration
docker compose --env-file .env.production -f docker-compose.prod.yaml up -d mysql minio server worker
```

## Standalone Mode

Standalone runs migration, server, and worker in one process. This is useful for small deployments:

```bash
docker compose --env-file .env.production -f docker-compose.prod.yaml --profile standalone up -d mysql minio standalone
```

## Storage

The production template uses MySQL for metadata, MinIO/S3 for attachments, packages, LFS objects, and artifacts, and Docker volumes for Git repositories and search indexes.

For external S3-compatible storage, change:

```env
GITY_STORAGE__S3_BUCKET=
GITY_STORAGE__S3_REGION=
GITY_STORAGE__S3_ENDPOINT=
GITY_STORAGE__S3_ACCESS_KEY=
GITY_STORAGE__S3_SECRET_KEY=
GITY_STORAGE__S3_USE_PATH_STYLE=
```
