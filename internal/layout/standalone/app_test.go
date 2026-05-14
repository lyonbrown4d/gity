package standalone_test

import (
	"context"
	"testing"
	"time"

	"github.com/DaiYuANg/gity/internal/layout/standalone"
	"github.com/DaiYuANg/gity/internal/testutil"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dix"
)

func TestStandaloneAppValidate(t *testing.T) {
	testutil.SetRuntimeEnv(t, t.TempDir())
	if err := standalone.NewStandaloneApp().Validate(); err != nil {
		t.Fatalf("validate standalone app: %v", err)
	}
}

func TestStandaloneRuntimeUsesSubApps(t *testing.T) {
	app := standalone.NewStandaloneApp()
	subapps := app.SubApps()
	if subapps.Len() != 3 {
		t.Fatalf("expected standalone to declare 3 subapps, got %d", subapps.Len())
	}
	migration, migrationOK := subapps.Get(0)
	server, serverOK := subapps.Get(1)
	worker, workerOK := subapps.Get(2)
	if !migrationOK || !serverOK || !workerOK {
		t.Fatalf("expected standalone subapps to be indexable")
	}
	if migration.Name() != "migration" || server.Name() != "server" || worker.Name() != "worker" {
		t.Fatalf("unexpected standalone subapps: %s, %s, %s", migration.Name(), server.Name(), worker.Name())
	}
}

func TestStandaloneRuntimeStartsEnsureSchema(t *testing.T) {
	testutil.SetRuntimeEnv(t, t.TempDir())
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	runtime, err := standalone.NewStandaloneApp().Start(ctx)
	if err != nil {
		t.Fatalf("start standalone runtime: %v", err)
	}
	if runtime.Meta().Version == "" {
		t.Fatalf("expected standalone runtime version")
	}
	workerRuntime, ok := runtime.SubApp("worker")
	if !ok {
		t.Fatalf("resolve worker subapp runtime")
	}
	db, err := dix.ResolveAs[*dbx.DB](workerRuntime.Container())
	if err != nil {
		t.Fatalf("resolve database runtime: %v", err)
	}
	testutil.AssertTableExists(ctx, t, db, "schema_history")
	testutil.AssertTableExists(ctx, t, db, "project_iid_counters")
	testutil.AssertTableExists(ctx, t, db, "project_jobs")

	stopCtx, stopCancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer stopCancel()
	if err := runtime.Stop(stopCtx); err != nil {
		t.Fatalf("stop standalone runtime: %v", err)
	}
}
