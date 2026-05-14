// Package standalone assembles the standalone dix application.
package standalone

import (
	gitydebug "github.com/DaiYuANg/gity/internal/debug"
	migrationapp "github.com/DaiYuANg/gity/internal/layout/migration"
	serverapp "github.com/DaiYuANg/gity/internal/layout/server"
	"github.com/DaiYuANg/gity/internal/layout/worker"
	"github.com/arcgolabs/dix"
)

func NewStandaloneApp() *dix.App {
	return dix.New(
		"gity-standalone",
		dix.Modules(
			gitydebug.Module(
				"cmd.standalone.meta",
				"gity-standalone",
				"Gity standalone runtime with migration, server, and worker sub-apps",
			),
		),
		dix.LifecycleConcurrency(4),
		dix.RecentEvents(512),
		dix.SubApps(
			migrationapp.NewSubApp(),
			serverapp.NewSubApp(),
			worker.NewSubApp(),
		),
	)
}
