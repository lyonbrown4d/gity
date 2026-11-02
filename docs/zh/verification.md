<div lang="zh-CN">

[简体中文](verification.md) · [English](../en/verification.md)

# 当前验证状态

本页记录已完成的检查与实际验收范围。Debian standalone 完整黑盒已通过；其余组合未获得完整验收结论。源码检查、编译、源码测试、容器专项与完整镜像验收分别记录，构建成功不等于功能验收完成。

## 已完成的源码测试

| 检查 | 当前已完成运行 | 证据 |
| --- | --- | --- |
| 工作区测试 | **332 项通过、0 项失败、13 项默认忽略** | `target/production-blackbox/workspace-tests-final.log`；按全部测试摘要汇总。 |
| PostgreSQL 18 业务回归 | **81 项通过** | `target/production-blackbox/database-regression.log` 中 PostgreSQL 子测试段。 |
| MySQL 8.4 业务回归 | **81 项通过** | `target/production-blackbox/mysql-regression-final.log`。 |
| PostgreSQL/MySQL 最新迁移结构契约 | **2 项通过** | `target/production-blackbox/migration-databases.log`。 |
| PostgreSQL/MySQL 合并锁专项 | **各 7 项通过** | `cargo test -p gity-test-support --test merge_guard_databases -- --ignored --test-threads=1`；独立 Testcontainers，覆盖恢复、abort 和 REST 引用隔离。 |
| GNU tar 元数据恢复专项 | **1 项通过** | `target/production-blackbox/backup-metadata-tests.log`。 |
| 真实 Docker OCI 认证握手专项 | **1 项通过** | 独立运行的 `gateway::docker_handshake_tests::real_docker_requires_authenticated_ping_for_login_and_challenge_for_push`；结果来自工具会话，未单独归档日志。 |

工作区集合包含类型化仓储、权限隔离、业务与审计事务、并发编号分配、流水线去重、Runner 租约、LFS、Git HTTP/SSH、个人 SSH 公钥、MR 合并恢复、备份清单和压力工具的单元/进程检查。最终普通工作区重跑已通过；完整容器验收仍以逐组黑盒结果为准。

81 项来自本轮较早完成的业务集合；收尾新增的数据库合并锁回归单独列出，未再次执行整个双数据库业务集合。

双数据库回归通过 `GITY_TEST_DATABASE_URL` 切换支持该夹具的测试。自建 SQLite 的传输测试和纯逻辑测试仍使用自己的资源；“81 项通过”不表示每个测试都改用了 PostgreSQL/MySQL。各容器和测试库独立创建，不使用保留环境数据。数据库回归命令为：

```sh
cargo test -p gity-test-support --test database_regression --locked -- --ignored --test-threads=1
```

13 项默认忽略包括实体容器契约 2 项、迁移容器契约 2 项、业务数据库入口 2 项、合并锁数据库入口 2 项，以及 OCI Docker 握手、Docker Hub 上游解析、容器 Runner、GNU tar 元数据和显式本地归档校验专项各 1 项。其中迁移、业务数据库入口、合并锁数据库入口、OCI 握手、元数据和显式本地归档校验专项已有单独运行证据；这些结果不改变默认工作区日志中的 ignored 数。实体容器契约、上游解析和容器 Runner 的独立执行结论尚未在本页确认，不能由其他场景推定通过。

OCI 握手专项可通过 `cargo test -p gity-oci --lib real_docker_requires_authenticated_ping --locked -- --ignored --nocapture` 单独执行；本轮使用 Docker 29.9.2，验证认证探测、错误登录拒绝、正确登录和真实 push。

GNU tar 专项检查数字 UID/GID、权限、扩展属性、ACL、文件 capabilities、SGID 和 whiteout 设备的往返。它验证归档元数据机制，不能替代数据库、Git、对象存储和秘密配置的完整恢复验收。

## 源码、编译与文档检查

- 最新格式检查、源码约束检查、第三方资产检查和 `git diff --check` 已通过。
- 默认工作区检查、最终 Clippy、Web WASM、mdBook 构建和双语链接检查均通过。
- 默认生产构建与测试构建必须分别核对，不能让 `gity-core/test-fixtures` 掩盖生产依赖错误。

## 镜像完整黑盒范围

| 部署形态 | 基础镜像 | 当前状态 |
| --- | --- | --- |
| Standalone | Debian | **43 项完整黑盒通过**，见[逐项证据](../evidence/standalone-debian.json)。 |
| Standalone | Alpine | 镜像已构建，未执行完整黑盒。 |
| 分布式组件 | Debian | 未完成；启动检查在 registry 配置校验处停止。 |
| 分布式组件 | Alpine | 未执行完整黑盒。 |

Debian standalone 的 43 项已覆盖真实 HTTP/SSH Git、个人与部署公钥权限、MR 中断恢复、Runner 容器任务、包协议、OCI/mirror、跨项目隔离、故障行为，以及完整离线备份→新项目恢复→重启→新任务执行。内部 Angos 的 hosted 对象校验只证明内部数据完整性，不能表述为 mirror 策略下的公网 hosted 认证验证。

Debian standalone 的一次容器快照为应用 74.18 MiB、DIND 282.5 MiB；这是功能测试后的单点观测，不是稳态内存、RSS 或容量结论。

应保留每组实际镜像 ID、平台、检查项、失败信息和恢复后的资源清理结果。一个部署形态或基础镜像通过，不能代替其他组合。入口见[镜像黑盒说明](deployment/images.md#镜像黑盒验收)和[部署恢复边界](production-deployment.md)。

## 本轮未执行与未覆盖范围

用户已取消本轮实际压力测试。保留 Git/SSH 与 Git REST 压力工具，但本轮不执行 k6 或真实客户端负载采样，不给出吞吐、负载延迟、稳态 CPU 或压力峰值内存结论。脚本解析、工具单元测试和计时 Gauge 不是实际压力结果。

arm64、多主机网络、真实生产数据恢复及无损自动故障切换未获得本页的验收结论。当前仅 Debian standalone 的完整恢复已获验收，其余组合须独立验证。

</div>
