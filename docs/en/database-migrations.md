<div lang="en">

[简体中文](../zh/database-migrations.md) · [English](database-migrations.md)

# Database migrations

Gity manages database structure in `crates/gity-migrations`. This is a reusable SeaORM migration library, not a standalone executable. The first migration, `m20261102_000001_latest_schema`, creates the complete schema required by the current Rust version from the fixed snapshot in `crates/gity-migrations/src/schema`. Future changes append Rust migrations; they do not reconstruct migration history from mutable entity definitions.

Migration versions and application timestamps are recorded in SeaORM's official `seaql_migrations` table. It stores only the migration version and `applied_at`; there is no custom checksum ledger. SeaORM does not provide a cross-process migration lock, so only one migration job may run against a database at a time. Standalone runs migrations before starting business modules. Distributed deployment uses the one-shot `gity-api migrate` command, and other services wait for it to succeed. Do not run multiple migration processes concurrently.

## Supported databases

The current baseline follows the latest Rust data model and supports SQLite, PostgreSQL, and MySQL. Older database schemas and their migration histories are not compatible inputs for this baseline.

**Initializing this migration baseline requires an empty database.** Databases already managed by this baseline can apply subsequent versions. Before upgrading, back up data that must be retained, explicitly stop the application, and have an operator clear or recreate the dedicated Gity database. Then start against the empty database. Gity never deletes a user database automatically and does not convert `rust_schema_history` into `seaql_migrations`. Unmanaged structures, unknown versions, or invalid migration records cause migration to fail. Do not reset the database with a global Compose volume cleanup that could delete volumes belonging to other services.

This baseline migration is irreversible. To roll back or recover data, stop the application and restore a backup. The migration's down operation fails; it cannot restore the previous schema. SQLite uses an outer transaction, while the framework uses a transaction for each PostgreSQL migration. MySQL DDL implicitly commits, so a failure can leave partial structure and cannot be assumed to roll back atomically. Inspect the database, then recreate the dedicated empty Gity database before retrying.

## Future changes

Append ordered, versioned Rust migrations in `crates/gity-migrations` for every schema change, with behavior for SQLite, PostgreSQL, and MySQL. Do not rewrite released migrations or use current SeaORM entities as the source of an older schema. Migration DDL uses typed definitions such as SeaQuery and `DeriveIden`; runtime does not parse hand-written SQL scripts or use a custom migration runner. Preserve existing relationship constraints and cascade behavior when rebuilding schema.

Entity primary keys continue to come from the shared application ID generator, not database auto-increment or default IDs. Migrations manage structure; they do not replace domain entity and repository rules.

Source locations: `crates/gity-migrations`, `crates/gity-core/src/database`, `apps/gity-api`, and the Standalone composition entry point.

</div>
