package core

import (
	"context"

	"github.com/arcgolabs/dbx"
	"github.com/samber/oops"
)

const projectDeletionStateMigration = "0014_project_deletion_state"

func ensureProjectDeletionState(ctx context.Context, tx *dbx.Tx) error {
	if err := ensureProjectDeletionColumn(ctx, tx, "status"); err != nil {
		return err
	}
	if err := ensureProjectDeletionColumn(ctx, tx, "deleted_at"); err != nil {
		return err
	}
	return ensureProjectStatusIndex(ctx, tx)
}

func ensureProjectDeletionColumn(ctx context.Context, tx *dbx.Tx, columnName string) error {
	exists, err := projectColumnExists(ctx, tx, columnName)
	if err != nil {
		return err
	}
	if exists {
		return nil
	}
	dialectName := migrationDialectName(tx)
	statement, ok := projectDeletionColumnStatement(dialectName, columnName)
	if !ok {
		return unsupportedMigrationDialect(projectDeletionStateMigration, dialectName)
	}
	if _, err := tx.ExecContext(ctx, statement); err != nil {
		return oops.In("persistence.schema").With("migration_version", projectDeletionStateMigration, "dialect", dialectName, "column", columnName).Wrapf(err, "add project deletion state column")
	}
	return nil
}

func ensureProjectStatusIndex(ctx context.Context, tx *dbx.Tx) error {
	dialectName := migrationDialectName(tx)
	switch dialectName {
	case "sqlite", "postgres":
		if _, err := tx.ExecContext(ctx, `CREATE INDEX IF NOT EXISTS "idx_projects_status" ON "projects" ("status")`); err != nil {
			return oops.In("persistence.schema").With("migration_version", projectDeletionStateMigration, "dialect", dialectName).Wrapf(err, "create project status index")
		}
		return nil
	case "mysql":
		return ensureProjectStatusIndexMySQL(ctx, tx, dialectName)
	default:
		return unsupportedMigrationDialect(projectDeletionStateMigration, dialectName)
	}
}

func ensureProjectStatusIndexMySQL(ctx context.Context, tx *dbx.Tx, dialectName string) error {
	exists, err := mysqlIndexExists(ctx, tx, projectDeletionStateMigration, "projects", "idx_projects_status")
	if err != nil {
		return err
	}
	if exists {
		return nil
	}
	if _, err := tx.ExecContext(ctx, "CREATE INDEX `idx_projects_status` ON `projects` (`status`)"); err != nil {
		return oops.In("persistence.schema").With("migration_version", projectDeletionStateMigration, "dialect", dialectName).Wrapf(err, "create project status index")
	}
	return nil
}

func projectColumnExists(ctx context.Context, tx *dbx.Tx, columnName string) (bool, error) {
	dialectName := migrationDialectName(tx)
	switch dialectName {
	case "sqlite":
		return sqliteTableColumnExists(ctx, tx, projectDeletionStateMigration, "projects", columnName)
	case "postgres":
		return postgresProjectColumnExists(ctx, tx, columnName)
	case "mysql":
		return mysqlProjectColumnExists(ctx, tx, columnName)
	default:
		return false, unsupportedMigrationDialect(projectDeletionStateMigration, dialectName)
	}
}

func postgresProjectColumnExists(ctx context.Context, tx *dbx.Tx, columnName string) (bool, error) {
	var count int
	err := tx.QueryRowContext(ctx, `
SELECT COUNT(*)
FROM information_schema.columns
WHERE table_schema = current_schema()
  AND table_name = ?
  AND column_name = ?`, "projects", columnName).Scan(&count)
	if err != nil {
		return false, oops.In("persistence.schema").With("migration_version", projectDeletionStateMigration, "table", "projects", "column", columnName).Wrapf(err, "check postgres project column")
	}
	return count > 0, nil
}

func mysqlProjectColumnExists(ctx context.Context, tx *dbx.Tx, columnName string) (bool, error) {
	var count int
	err := tx.QueryRowContext(ctx, `
SELECT COUNT(*)
FROM information_schema.columns
WHERE table_schema = DATABASE()
  AND table_name = ?
  AND column_name = ?`, "projects", columnName).Scan(&count)
	if err != nil {
		return false, oops.In("persistence.schema").With("migration_version", projectDeletionStateMigration, "table", "projects", "column", columnName).Wrapf(err, "check mysql project column")
	}
	return count > 0, nil
}

func projectDeletionColumnStatement(dialectName, columnName string) (string, bool) {
	switch dialectName {
	case "sqlite":
		return sqliteProjectDeletionColumnStatement(columnName)
	case "postgres":
		return postgresProjectDeletionColumnStatement(columnName)
	case "mysql":
		return mysqlProjectDeletionColumnStatement(columnName)
	default:
		return "", false
	}
}

func sqliteProjectDeletionColumnStatement(columnName string) (string, bool) {
	switch columnName {
	case "status":
		return `ALTER TABLE "projects" ADD COLUMN "status" TEXT NOT NULL DEFAULT 'active'`, true
	case "deleted_at":
		return `ALTER TABLE "projects" ADD COLUMN "deleted_at" TIMESTAMP NULL`, true
	default:
		return "", false
	}
}

func postgresProjectDeletionColumnStatement(columnName string) (string, bool) {
	switch columnName {
	case "status":
		return `ALTER TABLE "projects" ADD COLUMN "status" TEXT NOT NULL DEFAULT 'active'`, true
	case "deleted_at":
		return `ALTER TABLE "projects" ADD COLUMN "deleted_at" TIMESTAMP NULL`, true
	default:
		return "", false
	}
}

func mysqlProjectDeletionColumnStatement(columnName string) (string, bool) {
	switch columnName {
	case "status":
		return "ALTER TABLE `projects` ADD COLUMN `status` VARCHAR(32) NOT NULL DEFAULT 'active'", true
	case "deleted_at":
		return "ALTER TABLE `projects` ADD COLUMN `deleted_at` DATETIME NULL", true
	default:
		return "", false
	}
}
