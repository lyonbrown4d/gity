package app

import (
	"context"
	"fmt"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
	"github.com/arcgolabs/dix"
	coredb "github.com/DaiYuANg/gity/internal/repository/core"
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
