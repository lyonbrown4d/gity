package database

import (
	"context"

	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dix"
)

func Module() dix.Module {
	return dix.NewModule(
		"platform.database",
		dix.Description("Database runtime"),
		dix.Providers(
			dix.ProviderErr2(NewDatabase),
		),
		dix.Hooks(
			dix.OnStop(func(_ context.Context, db *dbx.DB) error {
				if db == nil {
					return nil
				}
				return db.Close()
			}),
		),
	)
}
