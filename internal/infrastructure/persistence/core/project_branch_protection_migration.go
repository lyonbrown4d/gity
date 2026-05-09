package core

import (
	"context"

	"github.com/arcgolabs/dbx"
	"github.com/samber/oops"
)

const projectBranchProtectionRulesMigration = "0015_project_branch_protection_rules"

func ensureProjectBranchProtectionRules(ctx context.Context, tx *dbx.Tx) error {
	for _, columnName := range projectBranchProtectionRuleColumns() {
		if err := ensureProjectBranchProtectionRuleColumn(ctx, tx, columnName); err != nil {
			return err
		}
	}
	if err := ensureProjectBranchProtectionRuleTypeIndex(ctx, tx); err != nil {
		return err
	}
	return ensureProjectBranchProtectionUniqueIndex(ctx, tx)
}

func projectBranchProtectionRuleColumns() []string {
	return []string{
		"rule_type",
		"push_access_level",
		"merge_access_level",
		"require_merge_request",
		"require_pipeline_success",
		"allow_force_push",
		"allow_delete",
	}
}

func ensureProjectBranchProtectionRuleColumn(ctx context.Context, tx *dbx.Tx, columnName string) error {
	exists, err := projectBranchProtectionColumnExists(ctx, tx, columnName)
	if err != nil {
		return err
	}
	if exists {
		return nil
	}
	dialectName := migrationDialectName(tx)
	statement, ok := projectBranchProtectionColumnStatement(dialectName, columnName)
	if !ok {
		return unsupportedMigrationDialect(projectBranchProtectionRulesMigration, dialectName)
	}
	if _, err := tx.ExecContext(ctx, statement); err != nil {
		return oops.In("persistence.schema").With("migration_version", projectBranchProtectionRulesMigration, "dialect", dialectName, "column", columnName).Wrapf(err, "add project branch protection rule column")
	}
	return nil
}

func ensureProjectBranchProtectionRuleTypeIndex(ctx context.Context, tx *dbx.Tx) error {
	dialectName := migrationDialectName(tx)
	switch dialectName {
	case "sqlite", "postgres":
		if _, err := tx.ExecContext(ctx, `CREATE INDEX IF NOT EXISTS "idx_project_branch_protections_rule_type" ON "project_branch_protections" ("rule_type")`); err != nil {
			return oops.In("persistence.schema").With("migration_version", projectBranchProtectionRulesMigration, "dialect", dialectName).Wrapf(err, "create project branch protection rule type index")
		}
		return nil
	case "mysql":
		return ensureProjectBranchProtectionRuleTypeIndexMySQL(ctx, tx, dialectName)
	default:
		return unsupportedMigrationDialect(projectBranchProtectionRulesMigration, dialectName)
	}
}

func ensureProjectBranchProtectionRuleTypeIndexMySQL(ctx context.Context, tx *dbx.Tx, dialectName string) error {
	exists, err := mysqlIndexExists(ctx, tx, projectBranchProtectionRulesMigration, "project_branch_protections", "idx_project_branch_protections_rule_type")
	if err != nil {
		return err
	}
	if exists {
		return nil
	}
	if _, err := tx.ExecContext(ctx, "CREATE INDEX `idx_project_branch_protections_rule_type` ON `project_branch_protections` (`rule_type`)"); err != nil {
		return oops.In("persistence.schema").With("migration_version", projectBranchProtectionRulesMigration, "dialect", dialectName).Wrapf(err, "create project branch protection rule type index")
	}
	return nil
}

func ensureProjectBranchProtectionUniqueIndex(ctx context.Context, tx *dbx.Tx) error {
	dialectName := migrationDialectName(tx)
	switch dialectName {
	case "sqlite", "postgres":
		return ensureProjectBranchProtectionUniqueIndexSQL(ctx, tx, dialectName)
	case "mysql":
		return ensureProjectBranchProtectionUniqueIndexMySQL(ctx, tx, dialectName)
	default:
		return unsupportedMigrationDialect(projectBranchProtectionRulesMigration, dialectName)
	}
}

func ensureProjectBranchProtectionUniqueIndexSQL(ctx context.Context, tx *dbx.Tx, dialectName string) error {
	if _, err := tx.ExecContext(ctx, `DROP INDEX IF EXISTS "ux_project_branch_protections_project_branch_unique"`); err != nil {
		return oops.In("persistence.schema").With("migration_version", projectBranchProtectionRulesMigration, "dialect", dialectName).Wrapf(err, "drop old project branch protection unique index")
	}
	if _, err := tx.ExecContext(ctx, `CREATE UNIQUE INDEX IF NOT EXISTS "ux_project_branch_protections_project_branch_unique" ON "project_branch_protections" ("project_id", "branch_name")`); err != nil {
		return oops.In("persistence.schema").With("migration_version", projectBranchProtectionRulesMigration, "dialect", dialectName).Wrapf(err, "create project branch protection project branch unique index")
	}
	return nil
}

func ensureProjectBranchProtectionUniqueIndexMySQL(ctx context.Context, tx *dbx.Tx, dialectName string) error {
	exists, err := mysqlIndexExists(ctx, tx, projectBranchProtectionRulesMigration, "project_branch_protections", "ux_project_branch_protections_project_branch_unique")
	if err != nil {
		return err
	}
	if exists {
		if _, err := tx.ExecContext(ctx, "DROP INDEX `ux_project_branch_protections_project_branch_unique` ON `project_branch_protections`"); err != nil {
			return oops.In("persistence.schema").With("migration_version", projectBranchProtectionRulesMigration, "dialect", dialectName).Wrapf(err, "drop old project branch protection unique index")
		}
	}
	if _, err := tx.ExecContext(ctx, "CREATE UNIQUE INDEX `ux_project_branch_protections_project_branch_unique` ON `project_branch_protections` (`project_id`, `branch_name`)"); err != nil {
		return oops.In("persistence.schema").With("migration_version", projectBranchProtectionRulesMigration, "dialect", dialectName).Wrapf(err, "create project branch protection project branch unique index")
	}
	return nil
}

func projectBranchProtectionColumnExists(ctx context.Context, tx *dbx.Tx, columnName string) (bool, error) {
	dialectName := migrationDialectName(tx)
	switch dialectName {
	case "sqlite":
		return sqliteTableColumnExists(ctx, tx, projectBranchProtectionRulesMigration, "project_branch_protections", columnName)
	case "postgres":
		return postgresProjectBranchProtectionColumnExists(ctx, tx, columnName)
	case "mysql":
		return mysqlProjectBranchProtectionColumnExists(ctx, tx, columnName)
	default:
		return false, unsupportedMigrationDialect(projectBranchProtectionRulesMigration, dialectName)
	}
}

func postgresProjectBranchProtectionColumnExists(ctx context.Context, tx *dbx.Tx, columnName string) (bool, error) {
	var count int
	err := tx.QueryRowContext(ctx, `
SELECT COUNT(*)
FROM information_schema.columns
WHERE table_schema = current_schema()
  AND table_name = ?
  AND column_name = ?`, "project_branch_protections", columnName).Scan(&count)
	if err != nil {
		return false, oops.In("persistence.schema").With("migration_version", projectBranchProtectionRulesMigration, "table", "project_branch_protections", "column", columnName).Wrapf(err, "check postgres project branch protection column")
	}
	return count > 0, nil
}

func mysqlProjectBranchProtectionColumnExists(ctx context.Context, tx *dbx.Tx, columnName string) (bool, error) {
	var count int
	err := tx.QueryRowContext(ctx, `
SELECT COUNT(*)
FROM information_schema.columns
WHERE table_schema = DATABASE()
  AND table_name = ?
  AND column_name = ?`, "project_branch_protections", columnName).Scan(&count)
	if err != nil {
		return false, oops.In("persistence.schema").With("migration_version", projectBranchProtectionRulesMigration, "table", "project_branch_protections", "column", columnName).Wrapf(err, "check mysql project branch protection column")
	}
	return count > 0, nil
}

func projectBranchProtectionColumnStatement(dialectName, columnName string) (string, bool) {
	switch dialectName {
	case "sqlite", "postgres":
		return sqlProjectBranchProtectionColumnStatement(columnName)
	case "mysql":
		return mysqlProjectBranchProtectionColumnStatement(columnName)
	default:
		return "", false
	}
}

func sqlProjectBranchProtectionColumnStatement(columnName string) (string, bool) {
	switch columnName {
	case "rule_type":
		return `ALTER TABLE "project_branch_protections" ADD COLUMN "rule_type" TEXT NOT NULL DEFAULT 'exact'`, true
	case "push_access_level":
		return `ALTER TABLE "project_branch_protections" ADD COLUMN "push_access_level" TEXT NOT NULL DEFAULT 'no_one'`, true
	case "merge_access_level":
		return `ALTER TABLE "project_branch_protections" ADD COLUMN "merge_access_level" TEXT NOT NULL DEFAULT 'maintainer'`, true
	case "require_merge_request":
		return `ALTER TABLE "project_branch_protections" ADD COLUMN "require_merge_request" INTEGER NOT NULL DEFAULT 1`, true
	case "require_pipeline_success":
		return `ALTER TABLE "project_branch_protections" ADD COLUMN "require_pipeline_success" INTEGER NOT NULL DEFAULT 0`, true
	case "allow_force_push":
		return `ALTER TABLE "project_branch_protections" ADD COLUMN "allow_force_push" INTEGER NOT NULL DEFAULT 0`, true
	case "allow_delete":
		return `ALTER TABLE "project_branch_protections" ADD COLUMN "allow_delete" INTEGER NOT NULL DEFAULT 0`, true
	default:
		return "", false
	}
}

func mysqlProjectBranchProtectionColumnStatement(columnName string) (string, bool) {
	switch columnName {
	case "rule_type":
		return "ALTER TABLE `project_branch_protections` ADD COLUMN `rule_type` VARCHAR(32) NOT NULL DEFAULT 'exact'", true
	case "push_access_level":
		return "ALTER TABLE `project_branch_protections` ADD COLUMN `push_access_level` VARCHAR(32) NOT NULL DEFAULT 'no_one'", true
	case "merge_access_level":
		return "ALTER TABLE `project_branch_protections` ADD COLUMN `merge_access_level` VARCHAR(32) NOT NULL DEFAULT 'maintainer'", true
	case "require_merge_request":
		return "ALTER TABLE `project_branch_protections` ADD COLUMN `require_merge_request` INTEGER NOT NULL DEFAULT 1", true
	case "require_pipeline_success":
		return "ALTER TABLE `project_branch_protections` ADD COLUMN `require_pipeline_success` INTEGER NOT NULL DEFAULT 0", true
	case "allow_force_push":
		return "ALTER TABLE `project_branch_protections` ADD COLUMN `allow_force_push` INTEGER NOT NULL DEFAULT 0", true
	case "allow_delete":
		return "ALTER TABLE `project_branch_protections` ADD COLUMN `allow_delete` INTEGER NOT NULL DEFAULT 0", true
	default:
		return "", false
	}
}
