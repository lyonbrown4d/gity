package core

import (
	"context"
	"fmt"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	identity "github.com/DaiYuANg/gity/internal/domain/identity"
	issuedomain "github.com/DaiYuANg/gity/internal/domain/issue"
	lfsdomain "github.com/DaiYuANg/gity/internal/domain/lfs"
	mergedomain "github.com/DaiYuANg/gity/internal/domain/merge"
	namespacedomain "github.com/DaiYuANg/gity/internal/domain/namespace"
	packagedomain "github.com/DaiYuANg/gity/internal/domain/packageregistry"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	wikidomain "github.com/DaiYuANg/gity/internal/domain/wiki"
	setx "github.com/arcgolabs/collectionx/set"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/schemamigrate"
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
	return []Migration{
		{
			Version: "0001_core",
			Name:    "bootstrap core namespace project auth schema",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := schemamigrate.AutoMigrate(ctx, tx, identity.UserSchema, namespacedomain.NamespaceSchema, projectdomain.ProjectSchema, namespacedomain.NamespaceMemberSchema, identity.UserAccessTokenSchema)
				return err
			},
		},
		{
			Version: "0002_project_issues",
			Name:    "add project issues comments and attachments",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := schemamigrate.AutoMigrate(ctx, tx, issuedomain.ProjectIssueSchema, issuedomain.ProjectIssueCommentSchema, issuedomain.ProjectIssueAttachmentSchema)
				return err
			},
		},
		{
			Version: "0003_project_merge_requests",
			Name:    "add project merge requests",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := schemamigrate.AutoMigrate(ctx, tx, mergedomain.ProjectMergeRequestSchema)
				return err
			},
		},
		{
			Version: "0004_project_packages",
			Name:    "add project package registry",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := schemamigrate.AutoMigrate(ctx, tx, packagedomain.ProjectPackageSchema, packagedomain.ProjectPackageVersionSchema, packagedomain.ProjectPackageFileSchema)
				return err
			},
		},
		{
			Version: "0005_project_lfs",
			Name:    "add project git lfs objects",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := schemamigrate.AutoMigrate(ctx, tx, lfsdomain.ProjectLFSObjectSchema)
				return err
			},
		},
		{
			Version: "0006_project_lfs_locks",
			Name:    "add project git lfs locks",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := schemamigrate.AutoMigrate(ctx, tx, lfsdomain.ProjectLFSLockSchema)
				return err
			},
		},
		{
			Version: "0007_project_jobs",
			Name:    "add project jobs",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := schemamigrate.AutoMigrate(ctx, tx, cidomain.ProjectJobSchema)
				return err
			},
		},
		{
			Version: "0008_project_wiki_pages",
			Name:    "add project wiki pages",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := schemamigrate.AutoMigrate(ctx, tx, wikidomain.ProjectWikiPageSchema)
				return err
			},
		},
		{
			Version: "0009_project_runners",
			Name:    "add project runners",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := schemamigrate.AutoMigrate(ctx, tx, cidomain.ProjectRunnerSchema)
				return err
			},
		},
		{
			Version: "0010_project_pipelines",
			Name:    "add project pipelines",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := schemamigrate.AutoMigrate(ctx, tx, cidomain.ProjectPipelineSchema, cidomain.ProjectPipelineJobSchema)
				return err
			},
		},
		{
			Version: "0011_project_job_logs_artifacts",
			Name:    "add project job logs and artifacts",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := schemamigrate.AutoMigrate(ctx, tx, cidomain.ProjectJobLogSchema, cidomain.ProjectJobArtifactSchema)
				return err
			},
		},
		{
			Version: "0012_project_branch_protections",
			Name:    "add project branch protections",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := schemamigrate.AutoMigrate(ctx, tx, projectdomain.ProjectBranchProtectionSchema)
				return err
			},
		},
	}
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

func appliedMigrationSet(ctx context.Context, db *dbx.DB) (*setx.Set[string], error) {
	rows, err := db.QueryContext(ctx, `SELECT version FROM schema_migrations`)
	if err != nil {
		return nil, fmt.Errorf("list applied migrations: %w", err)
	}
	defer rows.Close()
	applied := setx.NewSet[string]()
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

func applyMigration(ctx context.Context, db *dbx.DB, migration Migration) error {
	tx, err := db.BeginTx(ctx, nil)
	if err != nil {
		return fmt.Errorf("begin migration %s: %w", migration.Version, err)
	}
	committed := false
	defer func() {
		if !committed {
			_ = tx.RollbackContext(ctx)
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
