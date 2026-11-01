# Docker 集成环境

从仓库根目录执行。端口只绑定本机，数据库与 SeaweedFS 使用独立命名卷。
示例凭据仅供本机验证，可通过 Compose 中的 `INTEGRATION_*` 环境变量覆盖。

```sh
docker compose -f deploy/integration/compose.yaml up -d
docker compose -f deploy/integration/compose.yaml ps -a
docker compose -f deploy/integration/compose.yaml logs s3-init
```

PostgreSQL 通过 `pg_isready` 检查就绪。SeaweedFS `mini` 初始化 S3 凭据及私有 `gity` 桶；
`s3-init` 通过已签名的 S3 请求确认桶可访问，最多重试 30 次，成功后退出。
SeaweedFS 使用固定的官方镜像标签，配置依据[官方 mini 文档](https://github.com/seaweedfs/seaweedfs/wiki/Quick-Start-with-weed-mini)。

需要 MySQL 或可观测性服务时启用相应 profile：

```sh
docker compose -f deploy/integration/compose.yaml --profile mysql --profile observability up -d
```

| 服务 | 本机地址 |
| --- | --- |
| PostgreSQL | `localhost:5433` |
| MySQL | `localhost:3307` |
| SeaweedFS S3 | `http://localhost:9000` |
| SeaweedFS Admin | `http://localhost:9001` |
| OTLP HTTP | `http://localhost:4318/v1/traces` |
| Collector 健康检查 | `http://localhost:13133` |
| Tempo 就绪检查 | `http://localhost:3200/ready` |
| Prometheus | `http://localhost:9091` |
| Grafana | `http://localhost:3000` |

Grafana 默认凭据为 `gity` / `gity-observability-local-only`。

## 启动应用

```sh
cp deploy/integration/gity.toml.example deploy/integration/gity.local.toml
cargo run -p gity-server -- --config deploy/integration/gity.local.toml standalone
```

该示例启用 PostgreSQL、S3 和 OTLP，应同时启动 `observability` profile；不收集链路时，
将本地配置的 `observability.otlp_endpoint` 改为空字符串。
Prometheus 通过 `host.docker.internal:9090` 抓取应用独立的指标监听端口。

MySQL 应用配置使用 `driver = "mysql"` 以及
`dsn = "mysql://gity:gity-local-only@localhost:3307/gity"`。
SQLite 回归直接使用临时数据库，不需要容器。

数据库回归可指定本机服务，测试创建独立的随机数据库：

```sh
GITY_TEST_DATABASE_URL=postgres://gity:gity-local-only@localhost:5433/gity cargo test -p gity-server
```

MySQL 回归需要可创建临时数据库的账号，例如此集成环境的本机 root 账号。
测试保留随机数据库供排查。不要将测试连接指向业务数据库。

大批量迁移回归可启用独立的 `ci` profile。它使用 tmpfs 保存一次性测试库，避免反复执行 DDL 时争用开发库磁盘；不会改变常规 PostgreSQL/MySQL 的持久化配置。停止测试容器会丢弃这些临时数据库，不提供重启后持久性。

```sh
docker compose -f deploy/integration/compose.yaml --profile ci up -d --wait postgres-test mysql-test
GITY_TEST_DATABASE_URL=postgres://gity:gity-test-db-only@localhost:5434/gity cargo test -p gity-server -- --test-threads=1
GITY_TEST_DATABASE_URL=mysql://root:gity-test-root-only@localhost:3308/gity cargo test -p gity-server -- --test-threads=1
docker compose -f deploy/integration/compose.yaml --profile ci stop postgres-test mysql-test
```

测试进程串行建立独立数据库；并发领取和租约竞争仍在对应测试内部执行。

重复运行前，先等待上一轮测试进程全部退出，再重建这两个一次性容器：

```sh
docker compose -f deploy/integration/compose.yaml --profile ci up -d --wait --no-deps --force-recreate postgres-test mysql-test
```

随机测试库会保留到容器停止，连续多轮可能耗尽各容器的 1 GiB tmpfs。
出现 `No space left on device` 时，也在测试全部结束后执行上述命令，再重新运行测试。
该命令只重建 `postgres-test`（5434）和 `mysql-test`（3308），丢弃其中的临时测试库；
不重建开发数据库（5433、3307），也不删除命名卷。不要在另一个测试进程仍运行时执行。

## 停止

```sh
docker compose -f deploy/integration/compose.yaml --profile mysql --profile observability down
```

`down` 保留数据卷。明确需要清空全部集成数据时，再追加 `--volumes`。
本目录的单节点配置和示例密码仅用于集成环境。
