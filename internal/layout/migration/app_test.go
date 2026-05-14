package migration_test

import (
	"context"
	"testing"
	"time"

	migrationapp "github.com/DaiYuANg/gity/internal/layout/migration"
	"github.com/DaiYuANg/gity/internal/testutil"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dix"
)

func TestMigrationAppValidate(t *testing.T) {
	testutil.SetRuntimeEnv(t, t.TempDir())
	if err := migrationapp.NewApp().Validate(); err != nil {
		t.Fatalf("validate migration app: %v", err)
	}
}

func TestMigrationRuntimeStartsEnsureSchema(t *testing.T) {
	testutil.SetRuntimeEnv(t, t.TempDir())
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	runtime, err := migrationapp.NewApp().Start(ctx)
	if err != nil {
		t.Fatalf("start migration runtime: %v", err)
	}
	if runtime.Meta().Version == "" {
		t.Fatalf("expected migration runtime version")
	}
	db, err := dix.ResolveAs[*dbx.DB](runtime.Container())
	if err != nil {
		t.Fatalf("resolve database runtime: %v", err)
	}
	testutil.AssertTableExists(ctx, t, db, "schema_history")
	testutil.AssertTableExists(ctx, t, db, "project_iid_counters")
	testutil.AssertTableExists(ctx, t, db, "projects")

	stopCtx, stopCancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer stopCancel()
	if err := runtime.Stop(stopCtx); err != nil {
		t.Fatalf("stop migration runtime: %v", err)
	}
}
