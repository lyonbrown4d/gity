// Package bootstrap assembles application runtimes.
package bootstrap

import (
	"context"

	coredb "github.com/DaiYuANg/gity/internal/infrastructure/persistence/core"

	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dix"
	"github.com/samber/oops"
)

func RunMigration(ctx context.Context) (err error) {
	rt, err := NewMigrationApp().Build()
	if err != nil {
		return oops.In("bootstrap").Wrapf(err, "build migration app")
	}
	db, err := dix.ResolveAs[*dbx.DB](rt.Container())
	if err != nil {
		return oops.In("bootstrap").Wrapf(err, "resolve database runtime")
	}
	if db == nil {
		return oops.In("bootstrap").New("database runtime is not configured")
	}
	defer func() {
		if closeErr := db.Close(); closeErr != nil && err == nil {
			err = oops.In("bootstrap").Wrapf(closeErr, "close migration database")
		}
	}()
	if err := coredb.EnsureSchema(ctx, db); err != nil {
		return oops.In("bootstrap").Wrapf(err, "ensure database schema")
	}
	return nil
}
