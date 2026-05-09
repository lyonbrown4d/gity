package core

import (
	"context"
	"fmt"
	dbschema "github.com/DaiYuANg/gity/internal/infrastructure/persistence/db_schema"
	setx "github.com/arcgolabs/collectionx/set"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/schemamigrate"
	"github.com/samber/oops"
	"time"
)

type Migration struct {
	Version string
	Name    string
	Apply   func(context.Context, *dbx.Tx) error
}

func EnsureSchema(ctx context.Context, db *dbx.DB) error {
	if db == nil {
		return nil
	}
	if err := ensureMigrationTable(ctx, db); err != nil {
		return err
	}
	applied, err := appliedMigrationSet(ctx, db)
	if err != nil {
		return err
	}
	for _, migration := range migrations() {
		if applied.Contains(migration.Version) {
			continue
		}
		if err := applyMigration(ctx, db, migration); err != nil {
			return err
		}
	}
	return nil
}

func migrations() []Migration {
	items := coreMigrations()
	items = append(items, featureMigrations()...)
	items = append(items, ciMigrations()...)
	return append(items, fixMigrations()...)
}

func coreMigrations() []Migration {
	return []Migration{
		{
			Version: "0001_core",
			Name:    "bootstrap core organization project auth schema",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				return autoMigrate(ctx, tx, "0001_core", dbschema.UserSchema, dbschema.OrganizationSchema, dbschema.ProjectSchema, dbschema.OrganizationMemberSchema, dbschema.UserAccessTokenSchema)
			},
		},
		{
			Version: "0002_project_issues",
			Name:    "add project issues comments and attachments",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				return autoMigrate(ctx, tx, "0002_project_issues", dbschema.ProjectIssueSchema, dbschema.ProjectIssueCommentSchema, dbschema.ProjectIssueAttachmentSchema)
			},
		},
		{
			Version: "0003_project_merge_requests",
			Name:    "add project merge requests",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				return autoMigrate(ctx, tx, "0003_project_merge_requests", dbschema.ProjectMergeRequestSchema)
			},
		},
		{
			Version: "0004_project_packages",
			Name:    "add project package registry",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				return autoMigrate(ctx, tx, "0004_project_packages", dbschema.ProjectPackageSchema, dbschema.ProjectPackageVersionSchema, dbschema.ProjectPackageFileSchema)
			},
		},
	}
}

func featureMigrations() []Migration {
	return []Migration{
		{
			Version: "0005_project_lfs",
			Name:    "add project git lfs objects",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				return autoMigrate(ctx, tx, "0005_project_lfs", dbschema.ProjectLFSObjectSchema)
			},
		},
		{
			Version: "0006_project_lfs_locks",
			Name:    "add project git lfs locks",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				return autoMigrate(ctx, tx, "0006_project_lfs_locks", dbschema.ProjectLFSLockSchema)
			},
		},
		{
			Version: "0007_project_jobs",
			Name:    "add project jobs",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				return autoMigrate(ctx, tx, "0007_project_jobs", dbschema.ProjectJobSchema)
			},
		},
		{
			Version: "0008_project_wiki_pages",
			Name:    "add project wiki pages",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				return autoMigrate(ctx, tx, "0008_project_wiki_pages", dbschema.ProjectWikiPageSchema)
			},
		},
	}
}

func ciMigrations() []Migration {
	return []Migration{
		{
			Version: "0009_project_runners",
			Name:    "add project runners",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				return autoMigrate(ctx, tx, "0009_project_runners", dbschema.ProjectRunnerSchema)
			},
		},
		{
			Version: "0010_project_pipelines",
			Name:    "add project pipelines",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				return autoMigrate(ctx, tx, "0010_project_pipelines", dbschema.ProjectPipelineSchema, dbschema.ProjectPipelineJobSchema)
			},
		},
		{
			Version: "0011_project_job_logs_artifacts",
			Name:    "add project job logs and artifacts",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				return autoMigrate(ctx, tx, "0011_project_job_logs_artifacts", dbschema.ProjectJobLogSchema, dbschema.ProjectJobArtifactSchema)
			},
		},
		{
			Version: "0012_project_branch_protections",
			Name:    "add project branch protections",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				return autoMigrate(ctx, tx, "0012_project_branch_protections", dbschema.ProjectBranchProtectionSchema)
			},
		},
	}
}

func fixMigrations() []Migration {
	return []Migration{
		{
			Version: "0013_project_wiki_page_slug_unique",
			Name:    "fix project wiki page project slug unique index",
			Apply:   fixProjectWikiPageSlugUniqueIndex,
		},
		{
			Version: "0014_project_deletion_state",
			Name:    "add project deletion state",
			Apply:   ensureProjectDeletionState,
		},
		{
			Version: "0015_project_branch_protection_rules",
			Name:    "add project branch protection rule fields",
			Apply:   ensureProjectBranchProtectionRules,
		},
		{
			Version: "0016_project_audit_events",
			Name:    "add project audit events",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				return autoMigrate(ctx, tx, "0016_project_audit_events", dbschema.ProjectAuditEventSchema)
			},
		},
	}
}

func autoMigrate(ctx context.Context, tx *dbx.Tx, migrationVersion string, schemas ...schemamigrate.Resource) error {
	if _, err := schemamigrate.AutoMigrate(ctx, tx, schemas...); err != nil {
		return oops.In("persistence.schema").With("migration_version", migrationVersion).Wrapf(err, "auto migrate schema")
	}
	return nil
}

func ensureMigrationTable(ctx context.Context, db *dbx.DB) error {
	_, err := db.ExecContext(ctx, `
CREATE TABLE IF NOT EXISTS schema_migrations (
    version TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TIMESTAMP NOT NULL
)`)
	if err != nil {
		return fmt.Errorf("ensure schema_migrations table: %w", err)
	}
	return nil
}

func appliedMigrationSet(ctx context.Context, db *dbx.DB) (applied *setx.Set[string], err error) {
	rows, err := db.QueryContext(ctx, `SELECT version FROM schema_migrations`)
	if err != nil {
		return nil, fmt.Errorf("list applied migrations: %w", err)
	}
	defer func() {
		if closeErr := rows.Close(); closeErr != nil {
			if err != nil {
				err = oops.In("persistence.schema").Wrapf(oops.Join(err, closeErr), "list applied migrations and close rows")
				return
			}
			err = oops.In("persistence.schema").Wrapf(closeErr, "close applied migration rows")
		}
	}()
	applied = setx.NewSet[string]()
	for rows.Next() {
		var version string
		if err := rows.Scan(&version); err != nil {
			return nil, fmt.Errorf("scan applied migration: %w", err)
		}
		applied.Add(version)
	}
	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("iterate applied migrations: %w", err)
	}
	return applied, nil
}

func applyMigration(ctx context.Context, db *dbx.DB, migration Migration) (err error) {
	tx, err := db.BeginTx(ctx, nil)
	if err != nil {
		return fmt.Errorf("begin migration %s: %w", migration.Version, err)
	}
	committed := false
	defer func() {
		if !committed {
			err = rollbackMigration(ctx, tx, migration, err)
		}
	}()
	if err := migration.Apply(ctx, tx); err != nil {
		return fmt.Errorf("apply migration %s: %w", migration.Version, err)
	}
	if _, err := tx.ExecContext(ctx, `INSERT INTO schema_migrations(version, name, applied_at) VALUES (?, ?, ?)`, migration.Version, migration.Name, time.Now().UTC()); err != nil {
		return fmt.Errorf("record migration %s: %w", migration.Version, err)
	}
	if err := tx.CommitContext(ctx); err != nil {
		return fmt.Errorf("commit migration %s: %w", migration.Version, err)
	}
	committed = true
	return nil
}

func rollbackMigration(ctx context.Context, tx *dbx.Tx, migration Migration, err error) error {
	rollbackErr := tx.RollbackContext(ctx)
	if rollbackErr == nil {
		return err
	}
	if err != nil {
		return oops.In("persistence.schema").With("migration_version", migration.Version).Wrapf(oops.Join(err, rollbackErr), "apply migration and rollback")
	}
	return oops.In("persistence.schema").With("migration_version", migration.Version).Wrapf(rollbackErr, "rollback migration")
}
