package core

import (
	"context"

	"github.com/arcgolabs/dbx"
)

func EnsureSchema(ctx context.Context, db *dbx.DB) error {
	return ensureSQLMigrations(ctx, db)
}
