<div lang="en">

[简体中文](../../zh/deployment/integration.md) · [English](integration.md)

# Docker Integration Environment

Run these commands from the repository root. Ports bind to localhost only; the database and SeaweedFS use separate named volumes.
Example credentials are for local verification only. Override them with the Compose `INTEGRATION_*` environment variables.

```sh
docker compose -f deploy/integration/compose.yaml up -d
docker compose -f deploy/integration/compose.yaml ps -a
docker compose -f deploy/integration/compose.yaml logs s3-init
```

PostgreSQL readiness is checked with `pg_isready`. SeaweedFS `mini` initializes the S3 credentials and private `gity` bucket;
`s3-init` verifies bucket access with a signed S3 request, retries up to 30 times, then exits on success.
SeaweedFS uses a pinned official image tag; configuration follows the [official mini documentation](https://github.com/seaweedfs/seaweedfs/wiki/Quick-Start-with-weed-mini)。

Enable the relevant profiles when you need MySQL or observability services:

```sh
docker compose -f deploy/integration/compose.yaml --profile mysql --profile observability up -d
```

| Service | Local address |
| --- | --- |
| PostgreSQL | `localhost:5433` |
| MySQL | `localhost:3307` |
| SeaweedFS S3 | `http://localhost:9000` |
| SeaweedFS Admin | `http://localhost:9001` |
| OTLP HTTP | `http://localhost:4318/v1/traces` |
| Collector health check | `http://localhost:13133` |
| Tempo readiness check | `http://localhost:3200/ready` |
| Prometheus | `http://localhost:9091` |
| Grafana | `http://localhost:3000` |

Grafana default credentials are `gity` / `gity-observability-local-only`。

## Start the Application

```sh
cp deploy/integration/gity.toml.example deploy/integration/gity.local.toml
cargo run -p gity-server -- --config deploy/integration/gity.local.toml standalone
```

This example enables PostgreSQL, S3, and OTLP, so also start the `observability` profile. To disable trace collection,
set `observability.otlp_endpoint` to an empty string in the local configuration.
Prometheus scrapes the application's separate metrics listener at  `host.docker.internal:9090`.

For MySQL, configure the application with `driver = "mysql"` and
`dsn = "mysql://gity:gity-local-only@localhost:3307/gity"`。
SQLite regression tests use a temporary database directly and do not require a container.

Database regression tests can target a local service and create a separate random database:

```sh
GITY_TEST_DATABASE_URL=postgres://gity:gity-local-only@localhost:5433/gity cargo test -p gity-server
```

MySQL regression tests require an account that can create temporary databases, such as the local root account in this integration environment.
Tests retain random databases for investigation. Do not point test connections at a business database.

For large migration regression runs, enable the separate `ci` profile. It stores disposable test databases on tmpfs to avoid disk contention with development databases during repeated DDL execution. This does not change the persistence configuration for regular PostgreSQL/MySQL services. Stopping the test containers discards these temporary databases; they do not persist across restarts.

```sh
docker compose -f deploy/integration/compose.yaml --profile ci up -d --wait postgres-test mysql-test
GITY_TEST_DATABASE_URL=postgres://gity:gity-test-db-only@localhost:5434/gity cargo test -p gity-server -- --test-threads=1
GITY_TEST_DATABASE_URL=mysql://root:gity-test-root-only@localhost:3308/gity cargo test -p gity-server -- --test-threads=1
docker compose -f deploy/integration/compose.yaml --profile ci stop postgres-test mysql-test
```

The test process creates isolated databases serially; concurrent claims and lease contention are still exercised within the relevant tests.

Before rerunning, wait for all processes from the previous test run to exit, then recreate these two disposable containers:

```sh
docker compose -f deploy/integration/compose.yaml --profile ci up -d --wait --no-deps --force-recreate postgres-test mysql-test
```

Random test databases remain until the containers stop, so repeated runs may exhaust each container's 1 GiB tmpfs.
If you see `No space left on device`, run the command above after all tests have finished, then rerun the tests.
This command recreates only `postgres-test` (5434) and `mysql-test` (3308), discarding their temporary test databases;
it does not recreate the development databases (5433, 3307) or delete named volumes. Do not run it while another test process is active.

## Stop

```sh
docker compose -f deploy/integration/compose.yaml --profile mysql --profile observability down
```

`down` preserves data volumes. Add `--volumes` only when you explicitly want to clear all integration data.
The single-node configuration and example passwords in this directory are for integration environments only.

</div>
