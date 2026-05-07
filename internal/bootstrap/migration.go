package bootstrap

import (
	"context"
	"fmt"

	coredb "github.com/DaiYuANg/gity/internal/infrastructure/persistence/core"

	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dix"
)

func RunMigration(ctx context.Context) error {
	rt, err := NewMigrationApp().Build()
	if err != nil {
		return err
	}
	db, err := dix.ResolveAs[*dbx.DB](rt.Container())
	if err != nil {
		return fmt.Errorf("resolve database runtime: %w", err)
	}
	if db == nil {
		return fmt.Errorf("database runtime is not configured")
	}
	defer db.Close()
	return coredb.EnsureSchema(ctx, db)
}
