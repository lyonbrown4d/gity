package core

import (
	"context"

	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dix"
)

func Module() dix.Module {
	return dix.NewModule(
		"repository.core",
		dix.Description("Schema bootstrap"),
		dix.Hooks(
			dix.OnStart(func(ctx context.Context, db *dbx.DB) error {
				return EnsureSchema(ctx, db)
			}),
		),
	)
}
