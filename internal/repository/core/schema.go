package core

import (
	"context"
	"fmt"
	"time"

	"github.com/DaiYuANg/arcgo/dbx"
	"github.com/DaiYuANg/gity/internal/entity"
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
		if applied[migration.Version] {
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
				_, err := tx.AutoMigrate(ctx, entity.UserSchema, entity.NamespaceSchema, entity.ProjectSchema, entity.NamespaceMemberSchema, entity.UserAccessTokenSchema)
				return err
			},
		},
		{
			Version: "0002_project_issues",
			Name:    "add project issues comments and attachments",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := tx.AutoMigrate(ctx, entity.ProjectIssueSchema, entity.ProjectIssueCommentSchema, entity.ProjectIssueAttachmentSchema)
				return err
			},
		},
		{
			Version: "0003_project_merge_requests",
			Name:    "add project merge requests",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := tx.AutoMigrate(ctx, entity.ProjectMergeRequestSchema)
				return err
			},
		},
		{
			Version: "0004_project_packages",
			Name:    "add project package registry",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := tx.AutoMigrate(ctx, entity.ProjectPackageSchema, entity.ProjectPackageVersionSchema, entity.ProjectPackageFileSchema)
				return err
			},
		},
		{
			Version: "0005_project_lfs",
			Name:    "add project git lfs objects",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := tx.AutoMigrate(ctx, entity.ProjectLFSObjectSchema)
				return err
			},
		},
		{
			Version: "0006_project_lfs_locks",
			Name:    "add project git lfs locks",
			Apply: func(ctx context.Context, tx *dbx.Tx) error {
				_, err := tx.AutoMigrate(ctx, entity.ProjectLFSLockSchema)
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

func appliedMigrationSet(ctx context.Context, db *dbx.DB) (map[string]bool, error) {
	rows, err := db.QueryContext(ctx, `SELECT version FROM schema_migrations`)
	if err != nil {
		return nil, fmt.Errorf("list applied migrations: %w", err)
	}
	defer rows.Close()
	applied := make(map[string]bool)
	for rows.Next() {
		var version string
		if err := rows.Scan(&version); err != nil {
			return nil, fmt.Errorf("scan applied migration: %w", err)
		}
		applied[version] = true
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
