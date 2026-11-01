<div lang="en">

[简体中文](../../zh/deployment/compose.md) · [English](compose.md)

# Compose Component Deployment

`deploy/components/` contains two deployment files intended for a single Docker host:

- `standalone.compose.yaml` runs the complete Gity standalone application and starts a separate Docker-in-Docker (DinD) service for the Runner. The in-process Runner module supervises the Runner child process.
- `distributed.compose.yaml` splits API, Git server, Gateway, Registry, Worker, and two Git storage nodes, using PostgreSQL and SeaweedFS. The distributed Runner and DinD are started only by the `runner` profile.

For startup behavior, see [Standalone](../architecture/standalone.md) and [Distributed Components](../architecture/distributed.md)。

Neither layout is a multi-host deployment. Distributed Git replication is asynchronous after the primary acknowledges a write, and promotion is controlled; it does not provide Raft-based automatic failover. See [Deployment Overview](../production-deployment.md)。

## Prepare the Environment

Run from the repository root. Choose the Debian or Alpine environment template, copy it to the locally ignored `.env` file, then set separate values for the database, RPC, and S3. Values beginning with `replace-...` in the templates are placeholders and cannot be used for deployment as-is.

```sh
cp deploy/components/.env.example deploy/components/.env
# Or choose Alpine:
cp deploy/components/.env.alpine.example deploy/components/.env
chmod 600 deploy/components/.env
```

Use `openssl rand -hex 32` to generate separate values for `POSTGRES_PASSWORD`、`AUTHORITY_PEER_TOKEN`、the two Git node peer tokens, and the SeaweedFS access and secret keys. Fill in the Docker Hub upstream username and token only if the private image repository requires authentication; provide both or leave both blank. You do not need to set a real `GITY_DISTRIBUTED_RUNNER_TOKEN` before registering the Runner.

`GITY_OUTBOUND_HTTP_PROXY`, `GITY_OUTBOUND_HTTPS_PROXY`, and `GITY_OUTBOUND_NO_PROXY` set the standard `HTTP_PROXY`, `HTTPS_PROXY`, and `NO_PROXY` environment variables for Gity applications and DinD containers. Proxying is disabled by default in both environment templates; set the corresponding proxy URL when needed. The default `NO_PROXY` includes local addresses and Compose service names; preserve addresses that need direct access when customizing it. The proxy must be reachable from the container; do not use a host loopback address. Outbound proxying for image builds uses Docker standard `HTTP_PROXY` / `HTTPS_PROXY` build args separately.

Compose task containers and the Docker daemon must see the Runner working directory at the same absolute path. The template defaults to `/tmp/gity-runner-work`; create this directory before startup and assign it to container UID/GID `10001`:

```sh
sudo install -d -o 10001 -g 10001 /tmp/gity-runner-work
```

### Generate TLS and Secret Files

`generate-tls.sh` creates a local CA, server certificates for the API and both Git nodes, and client certificates for the API, Git server, and Worker. Certificate SANs include Compose service DNS names. Set `GITY_TLS_DIR` to choose the output directory. `prepare-secrets.sh` reads `.env` values from the process environment and generates Git node tokens, SeaweedFS S3 identity JSON, and Angos S3/mirror configuration. Set `GITY_SECRETS_DIR` to choose its output directory. Use absolute directories that Compose can bind-mount.

```sh
set -a
. deploy/components/.env
set +a
export GITY_TLS_DIR="$PWD/deploy/components/tls"
export GITY_SECRETS_DIR="$PWD/deploy/components/secrets"
sh deploy/components/generate-tls.sh
sh deploy/components/prepare-secrets.sh
sudo chown -R 10001:10001 "$GITY_TLS_DIR" "$GITY_SECRETS_DIR"
sudo chmod 0700 "$GITY_TLS_DIR" "$GITY_SECRETS_DIR"
sudo chmod 0600 "$GITY_TLS_DIR"/*.key
sudo chmod 0644 "$GITY_TLS_DIR"/*.crt
sudo chmod 0400 "$GITY_SECRETS_DIR"/*
```

Service containers read these files as UID/GID `10001`. Keep private keys at `0600`, S3, Angos, and node-token files at `0400`, and both directories at `0700`. Compose mounts the secrets directory read-only; do not commit generated files to Git. The local CA-issued certificates are only suitable for local Compose use; replace them with organization-managed CA certificates in production.

After changing the environment template, check the rendered Compose configuration. Redirecting the output avoids printing configuration values to the terminal:

```sh
docker compose --env-file deploy/components/.env \
  -f deploy/components/standalone.compose.yaml config > /dev/null
docker compose --env-file deploy/components/.env \
  -f deploy/components/distributed.compose.yaml config > /dev/null
```

## Start Standalone

Standalone uses SQLite and the local object directory from its configuration file. The Runner executes container jobs through the DinD service in the same Compose project. It publishes HTTP `8080`, Git SSH `2222`, and OCI `5000`:

```sh
docker compose \
  --env-file deploy/components/.env \
  -f deploy/components/standalone.compose.yaml \
  up -d --build
docker compose \
  --env-file deploy/components/.env \
  -f deploy/components/standalone.compose.yaml \
  ps
```

Open `http://localhost:8080`; the health check is at `/api/health`. The Docker daemon uses TLS; the Docker socket is not mounted into the Gity container. DinD requires a privileged container and must run on an isolated Docker host.

## Start the Distributed Components

Do not enable the `runner` profile when starting the base stack. Compose waits for PostgreSQL, SeaweedFS, and `seaweed-buckets` to become ready, then runs migrations once and starts the services. The initialization job uses the same S3 identity to create the `gity-artifacts` and `gity-oci` buckets:

```sh
docker compose \
  --env-file deploy/components/.env \
  -f deploy/components/distributed.compose.yaml \
  up -d --build
docker compose \
  --env-file deploy/components/.env \
  -f deploy/components/distributed.compose.yaml \
  ps
```

Gateway publishes HTTP `8080` and Git SSH `2222`; OCI is published on `5000`. Package clients use the Gateway paths `/api/v1/projects/{id}/packages/...` or `/api/v1/repos/{id}/packages/...`. Gateway forwards package requests to Registry internal port `8082`; the Registry OCI service listens on internal port `8081`. Authority RPC `9100` and Git storage RPC `9101` are not published to the host.

Gateway defaults to HTTP. If an external TLS proxy terminates HTTPS, change `deploy/components/config/gateway.toml` `public_scheme` to `https` so Gateway forwards the correct public scheme; `Host` is forwarded with the request.

### Start the Distributed Runner in Two Stages

First register the Runner in Gity project settings and obtain a project-scoped Runner token. Provide the token only in the interactive environment for the command below; input is not echoed or written to `.env`:

```sh
read -r -s -p "Runner token: " GITY_DISTRIBUTED_RUNNER_TOKEN
printf '\n'
export GITY_DISTRIBUTED_RUNNER_TOKEN
docker compose \
  --env-file deploy/components/.env \
  -f deploy/components/distributed.compose.yaml \
  --profile runner up -d --build docker runner
unset GITY_DISTRIBUTED_RUNNER_TOKEN
```

The command uses Bash silent input.  The Runner connects to `http://gateway:8080` and connects to DinD at `tcp://docker:2376` over TLS. Runner and DinD mount the working directory at the same absolute path so the Docker daemon container can find the source and scripts.

## Verify and Stop

A passing Compose health check means only that the corresponding process readiness probe succeeded.After startup, also check `http://localhost:8080/api/health` and verify data paths with actual Git clone/push, package upload/download, and OCI push/pull operations. Git nodes have no host-published ports.

Use the same `-f` and `--env-file` arguments when running `docker compose down` to stop services while preserving named volumes. Do not use `down -v`, which deletes the database, repositories, object storage, and Docker data volumes.

</div>
