# Production Deployment

The beta release publishes separate Docker images for each runtime:

- `ghcr.io/lyonbrown4d/gity-server`
- `ghcr.io/lyonbrown4d/gity-migration`
- `ghcr.io/lyonbrown4d/gity-search-index`
- `ghcr.io/lyonbrown4d/gity-worker`
- `ghcr.io/lyonbrown4d/gity-standalone`
- `ghcr.io/lyonbrown4d/gity-runner`

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

## Search Index Maintenance

For large bulk imports, run the standalone image or `gity-search-index` job with your
runtime configuration:

```bash
docker run --rm \
  --env-file .env.production \
  -v repo_data:/var/lib/gity/repos \
  -v search_index:/var/lib/gity/search-index \
  ghcr.io/lyonbrown4d/gity-search-index:latest
```

The image accepts the same `--project-id` and `--all` flags as local usage.

For external S3-compatible storage, change:

```env
GITY_STORAGE__S3_BUCKET=
GITY_STORAGE__S3_REGION=
GITY_STORAGE__S3_ENDPOINT=
GITY_STORAGE__S3_ACCESS_KEY=
GITY_STORAGE__S3_SECRET_KEY=
GITY_STORAGE__S3_USE_PATH_STYLE=
```
