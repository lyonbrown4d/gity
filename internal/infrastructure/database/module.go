package database

import (
	"context"
	"time"

	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dix"
	"github.com/samber/oops"
)

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.database",
		dix.Description("Database runtime"),
		dix.Providers(
			dix.ProviderErr2(NewDatabase, dix.Eager()),
		),
		dix.Setups(
			dix.SetupContainer(registerHealthChecks),
		),
		dix.Hooks(
			dix.OnStop(func(_ context.Context, db *dbx.DB) error {
				if db == nil {
					return nil
				}
				return db.Close()
			},
				dix.LifecycleName("database.close"),
				dix.LifecyclePriority(10),
				dix.LifecycleTimeout(5*time.Second),
			),
		),
	)
}

func registerHealthChecks(container *dix.Container) error {
	check := databaseHealthCheck(container)
	container.RegisterHealthCheck("database", check)
	container.RegisterReadinessCheck("database", check)
	return nil
}

func databaseHealthCheck(container *dix.Container) dix.HealthCheckFunc {
	return func(ctx context.Context) error {
		db, err := dix.ResolveAsContext[*dbx.DB](ctx, container)
		if err != nil {
			return oops.In("database").Wrapf(err, "resolve database health check")
		}
		if db == nil || db.SQLDB() == nil {
			return oops.In("database").New("database is not initialized")
		}
		if err := db.SQLDB().PingContext(ctx); err != nil {
			return oops.In("database").Wrapf(err, "ping database")
		}
		return nil
	}
}
