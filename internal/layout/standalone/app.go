// Package standalone assembles the standalone dix application.
package standalone

import (
	"github.com/arcgolabs/dix"
	gitydebug "github.com/lyonbrown4d/gity/internal/debug"
	migrationapp "github.com/lyonbrown4d/gity/internal/layout/migration"
	serverapp "github.com/lyonbrown4d/gity/internal/layout/server"
	"github.com/lyonbrown4d/gity/internal/layout/worker"
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
