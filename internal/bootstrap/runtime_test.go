package bootstrap

import (
	"context"
	"path/filepath"
	"testing"
	"time"

	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dix"
)

func TestRuntimeAppsValidate(t *testing.T) {
	cases := map[string]func() interface{ Validate() error }{
		"migration":  func() interface{ Validate() error } { return NewMigrationApp() },
		"server":     func() interface{ Validate() error } { return NewServerApp() },
		"worker":     func() interface{ Validate() error } { return NewWorkerApp() },
		"standalone": func() interface{ Validate() error } { return NewStandaloneApp() },
	}
	for name, factory := range cases {
		name := name
		factory := factory
		t.Run(name, func(t *testing.T) {
			tempDir := t.TempDir()
			setRuntimeTestEnv(t, tempDir)
			if err := factory().Validate(); err != nil {
				t.Fatalf("validate runtime app: %v", err)
			}
		})
	}
}

func TestServerRuntimeStarts(t *testing.T) {
	tempDir := t.TempDir()
	setRuntimeTestEnv(t, tempDir)

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	runtime, err := NewServerApp().Start(ctx)
	if err != nil {
		t.Fatalf("start server runtime: %v", err)
	}
	stopCtx, stopCancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer stopCancel()
	if err := runtime.Stop(stopCtx); err != nil {
		t.Fatalf("stop server runtime: %v", err)
	}
}

func TestRuntimeStartsEnsureSchema(t *testing.T) {
	cases := map[string]func() interface {
		Start(context.Context) (*dix.Runtime, error)
	}{
		"server": func() interface {
			Start(context.Context) (*dix.Runtime, error)
		} {
			return NewServerApp()
		},
		"worker": func() interface {
			Start(context.Context) (*dix.Runtime, error)
		} {
			return NewWorkerApp()
		},
		"standalone": func() interface {
			Start(context.Context) (*dix.Runtime, error)
		} {
			return NewStandaloneApp()
		},
	}
	for name, factory := range cases {
		name := name
		factory := factory
		t.Run(name, func(t *testing.T) {
			tempDir := t.TempDir()
			setRuntimeTestEnv(t, tempDir)

			ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			defer cancel()

			runtime, err := factory().Start(ctx)
			if err != nil {
				t.Fatalf("start runtime app: %v", err)
			}
			db, err := dix.ResolveAs[*dbx.DB](runtime.Container())
			if err != nil {
				t.Fatalf("resolve database runtime: %v", err)
			}
			assertTableExists(t, ctx, db, "project_jobs")
			assertTableExists(t, ctx, db, "project_pipelines")
			assertTableExists(t, ctx, db, "schema_migrations")

			stopCtx, stopCancel := context.WithTimeout(context.Background(), 5*time.Second)
			defer stopCancel()
			if err := runtime.Stop(stopCtx); err != nil {
				t.Fatalf("stop runtime app: %v", err)
			}
		})
	}
}

func setRuntimeTestEnv(t *testing.T, tempDir string) {
	t.Helper()
	t.Setenv("GITY_APP__ENVIRONMENT", "production")
	t.Setenv("GITY_HTTP__ADDRESS", "127.0.0.1:0")
	t.Setenv("GITY_DATABASE__DSN", "file:"+filepath.ToSlash(filepath.Join(tempDir, "gity.db"))+"?_pragma=foreign_keys(1)")
	t.Setenv("GITY_DATABASE__NODE_ID", "1")
	t.Setenv("GITY_GIT__REPO_ROOT", filepath.ToSlash(filepath.Join(tempDir, "repos")))
	t.Setenv("GITY_STORAGE__ROOT", filepath.ToSlash(filepath.Join(tempDir, "storage")))
	t.Setenv("GITY_WORKER__ENABLED", "false")
	t.Setenv("GITY_WORKER__POLL_INTERVAL_MILLIS", "10000")
}

func assertTableExists(t *testing.T, ctx context.Context, db *dbx.DB, tableName string) {
	t.Helper()
	var found string
	row := db.QueryRowContext(ctx, `SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?`, tableName)
	if err := row.Scan(&found); err != nil {
		t.Fatalf("expected table %s to exist: %v", tableName, err)
	}
	if found != tableName {
		t.Fatalf("expected table %s, got %s", tableName, found)
	}
}
