<div lang="zh-CN">

[简体中文](database-migrations.md) · [English](../en/database-migrations.md)

# 数据库迁移

Gity 使用 `crates/gity-migrations` 管理数据库结构。该 crate 是由应用复用的 SeaORM migration 库，不提供独立可执行程序。首个迁移 `m20261102_000001_latest_schema` 根据固定快照 `crates/gity-migrations/src/schema`，在空数据库上建立当前 Rust 版本所需的完整结构；后续结构变化追加新的 Rust 迁移，不从可变的实体定义重建历史迁移。

迁移版本和执行时间记录在 SeaORM 官方的 `seaql_migrations` 表中。该表只记录版本和 `applied_at`，不含自定义校验和账本。SeaORM 不提供跨进程迁移锁，因此同一数据库必须只运行一个迁移任务：Standalone 在启动业务组件前执行迁移；分布式部署使用一次性 `gity-api migrate` 命令，其他服务等待该任务成功后启动。不要让多个迁移进程并发运行。

## 适用数据库

当前基线以最新 Rust 数据结构为准，支持 SQLite、PostgreSQL 和 MySQL。旧数据库结构及其迁移历史不属于此基线的兼容输入。

**首次采用此迁移基线时必须使用空数据库。** 此基线管理的数据库可继续应用后续版本。 升级前备份需要保留的数据，并由运维人员明确停止应用、清空或重新创建专用的 Gity 数据库，再在空库上启动。Gity 不会自动删除用户数据库，也不会把 `rust_schema_history` 转换为 `seaql_migrations`；发现未受管结构、未知版本或无效迁移记录时，会拒绝继续。不要用会删除其他服务卷的全局 Compose 卷清理操作来重置数据库。

这一基线迁移不可逆。回退或恢复数据时，先停止应用并从备份恢复；迁移的 down 操作会失败，不能用它还原旧结构。SQLite 外层使用事务；PostgreSQL 由框架逐迁移使用事务。MySQL 的 DDL 会隐式提交，失败后可能留下部分结构，不能假设事务会原子回滚；先检查数据库状态，再重新创建空的专用 Gity 数据库后重试。

## 后续变更

每次数据库结构变化都应在 `crates/gity-migrations` 中追加有序、版本化的 Rust migration，并覆盖 SQLite、PostgreSQL 和 MySQL 所需行为。不要修改已发布迁移来重写历史，也不要把当前 SeaORM entities 当作旧版本 schema 的来源。迁移 DDL 使用 SeaQuery/`DeriveIden` 等类型化定义；运行时不解析手写 SQL 脚本，也不通过自建迁移 runner 执行。迁移重建结构时保留现有关系约束和级联行为。

新增实体主键仍由应用统一生成，不使用数据库自增或默认 ID。数据库迁移只负责结构，不替代领域实体及仓储规则。

源码位置：`crates/gity-migrations`、`crates/gity-core/src/database`、`apps/gity-api` 与 Standalone 组合入口。

</div>
