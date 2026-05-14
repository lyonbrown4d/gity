package server_test

import (
	"context"
	"testing"
	"time"

	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dix"
	serverapp "github.com/lyonbrown4d/gity/internal/layout/server"
	"github.com/lyonbrown4d/gity/internal/testutil"
)

func TestServerAppValidate(t *testing.T) {
	testutil.SetRuntimeEnv(t, t.TempDir())
	if err := serverapp.NewApp().Validate(); err != nil {
		t.Fatalf("validate server app: %v", err)
	}
}

func TestServerRuntimeStartsWithoutManagingSchema(t *testing.T) {
	testutil.SetRuntimeEnv(t, t.TempDir())
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	runtime, err := serverapp.NewApp().Start(ctx)
	if err != nil {
		t.Fatalf("start server runtime: %v", err)
	}
	if runtime.Meta().Version == "" {
		t.Fatalf("expected server runtime version")
	}
	db, err := dix.ResolveAs[*dbx.DB](runtime.Container())
	if err != nil {
		t.Fatalf("resolve database runtime: %v", err)
	}
	testutil.AssertTableMissing(ctx, t, db, "schema_history")
	testutil.AssertTableMissing(ctx, t, db, "projects")

	stopCtx, stopCancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer stopCancel()
	if err := runtime.Stop(stopCtx); err != nil {
		t.Fatalf("stop server runtime: %v", err)
	}
}
