package workerapp_test

import (
	"context"
	"testing"
	"time"

	workerapp "github.com/DaiYuANg/gity/cmd/workerapp"
	"github.com/DaiYuANg/gity/internal/testutil"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dix"
)

func TestWorkerAppValidate(t *testing.T) {
	testutil.SetRuntimeEnv(t, t.TempDir())
	if err := workerapp.NewApp().Validate(); err != nil {
		t.Fatalf("validate worker app: %v", err)
	}
}

func TestWorkerRuntimeStartsWithoutManagingSchema(t *testing.T) {
	testutil.SetRuntimeEnv(t, t.TempDir())
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	runtime, err := workerapp.NewApp().Start(ctx)
	if err != nil {
		t.Fatalf("start worker runtime: %v", err)
	}
	if runtime.Meta().Version == "" {
		t.Fatalf("expected worker runtime version")
	}
	db, err := dix.ResolveAs[*dbx.DB](runtime.Container())
	if err != nil {
		t.Fatalf("resolve database runtime: %v", err)
	}
	testutil.AssertTableMissing(ctx, t, db, "schema_migrations")
	testutil.AssertTableMissing(ctx, t, db, "project_jobs")

	stopCtx, stopCancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer stopCancel()
	if err := runtime.Stop(stopCtx); err != nil {
		t.Fatalf("stop worker runtime: %v", err)
	}
}
