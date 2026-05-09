// Package migrationapp assembles the database migration dix application.
package migrationapp

import (
	"context"

	"github.com/DaiYuANg/gity/internal/config"
	gitydebug "github.com/DaiYuANg/gity/internal/debug"
	"github.com/DaiYuANg/gity/internal/infrastructure/database"
	infralogger "github.com/DaiYuANg/gity/internal/infrastructure/logger"
	coredb "github.com/DaiYuANg/gity/internal/infrastructure/persistence/core"
	"github.com/arcgolabs/dix"
	"github.com/samber/oops"
)

// NewApp builds the database migration application.
func NewApp() *dix.App {
	return dix.New(
		"gity-migration",
		appOptions("cmd.migration.meta", "gity-migration", "Gity database migration runtime")...,
	)
}

// NewSubApp builds the migration sub-application used by standalone.
func NewSubApp() *dix.App {
	return dix.NewSubApp(
		"migration",
		appOptions("cmd.migration.meta", "migration", "Gity database migration subapp runtime")...,
	)
}

func appOptions(metaModuleName, appName, description string) []dix.AppOption {
	return []dix.AppOption{
		dix.UseLoggerErr1(infralogger.NewLogger),
		dix.Modules(
			gitydebug.Module(metaModuleName, appName, description),
			migrationRuntimeModule(),
		),
	}
}

func migrationRuntimeModule() dix.Module {
	return dix.NewModule(
		"cmd.migration",
		dix.Description("Migration command composition"),
		dix.Imports(
			config.Module(),
			database.Module(),
			coredb.Module(),
		),
	)
}

// Run starts the migration application and applies schema migrations.
func Run(ctx context.Context) (err error) {
	if ctx == nil {
		return oops.In("cmd.migration").New("migration context is required")
	}
	rt, err := NewApp().Start(ctx)
	if err != nil {
		return oops.In("cmd.migration").Wrapf(err, "start migration app")
	}
	defer func() {
		if stopErr := rt.Stop(context.WithoutCancel(ctx)); stopErr != nil && err == nil {
			err = oops.In("cmd.migration").Wrapf(stopErr, "stop migration app")
		}
	}()
	return nil
}
