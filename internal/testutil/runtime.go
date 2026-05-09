package testutil

import (
	"context"
	"database/sql"
	"errors"
	"path/filepath"
	"testing"

	"github.com/arcgolabs/dbx"
)

func SetRuntimeEnv(tb testing.TB, tempDir string) {
	tb.Helper()
	tb.Setenv("GITY_APP__ENVIRONMENT", "production")
	tb.Setenv("GITY_HTTP__ADDRESS", "127.0.0.1:0")
	tb.Setenv("GITY_DATABASE__DSN", "file:"+filepath.ToSlash(filepath.Join(tempDir, "gity.db"))+"?_pragma=foreign_keys(1)")
	tb.Setenv("GITY_DATABASE__NODE_ID", "1")
	tb.Setenv("GITY_GIT__REPO_ROOT", filepath.ToSlash(filepath.Join(tempDir, "repos")))
	tb.Setenv("GITY_STORAGE__ROOT", filepath.ToSlash(filepath.Join(tempDir, "storage")))
	tb.Setenv("GITY_WORKER__ENABLED", "false")
	tb.Setenv("GITY_WORKER__POLL_INTERVAL_MILLIS", "10000")
}

func AssertTableExists(ctx context.Context, tb testing.TB, db *dbx.DB, tableName string) {
	tb.Helper()
	var found string
	row := db.QueryRowContext(ctx, `SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?`, tableName)
	if err := row.Scan(&found); err != nil {
		tb.Fatalf("expected table %s to exist: %v", tableName, err)
	}
	if found != tableName {
		tb.Fatalf("expected table %s, got %s", tableName, found)
	}
}

func AssertTableMissing(ctx context.Context, tb testing.TB, db *dbx.DB, tableName string) {
	tb.Helper()
	var found string
	row := db.QueryRowContext(ctx, `SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?`, tableName)
	if err := row.Scan(&found); err == nil {
		tb.Fatalf("expected table %s to be missing", tableName)
	} else if !errors.Is(err, sql.ErrNoRows) {
		tb.Fatalf("check table %s is missing: %v", tableName, err)
	}
}
