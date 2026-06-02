package searchindex_test

import (
	"context"
	"testing"
	"time"

	"github.com/arcgolabs/dix"
	searchindexruntime "github.com/lyonbrown4d/gity/internal/infrastructure/search_index"
	searchindexapp "github.com/lyonbrown4d/gity/internal/layout/searchindex"
	"github.com/lyonbrown4d/gity/internal/testutil"
)

func TestSearchIndexAppValidate(t *testing.T) {
	testutil.SetRuntimeEnv(t, t.TempDir())
	if err := searchindexapp.NewApp().Validate(); err != nil {
		t.Fatalf("validate search index app: %v", err)
	}
}

func TestSearchIndexAppStartsWithoutManagingSchema(t *testing.T) {
	testutil.SetRuntimeEnv(t, t.TempDir())
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	runtime, err := searchindexapp.NewApp().Start(ctx)
	if err != nil {
		t.Fatalf("start search index app: %v", err)
	}
	if runtime.Meta().Version == "" {
		t.Fatalf("expected search index app version")
	}
	if _, err := dix.ResolveAs[*searchindexruntime.Service](runtime.Container()); err != nil {
		t.Fatalf("resolve search index service: %v", err)
	}

	stopCtx, stopCancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer stopCancel()
	if err := runtime.Stop(stopCtx); err != nil {
		t.Fatalf("stop search index app: %v", err)
	}
}
