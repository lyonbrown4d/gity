package main

import (
	migrationapp "github.com/DaiYuANg/gity/cmd/migrationapp"
	serverapp "github.com/DaiYuANg/gity/cmd/serverapp"
	workerapp "github.com/DaiYuANg/gity/cmd/workerapp"
	gitydebug "github.com/DaiYuANg/gity/internal/debug"
	"github.com/arcgolabs/dix"
)

func newStandaloneApp() *dix.App {
	return dix.New(
		"gity-standalone",
		dix.Modules(
			gitydebug.Module(
				"cmd.standalone.meta",
				"gity-standalone",
				"Gity standalone runtime with migration, server, and worker sub-apps",
			),
		),
		dix.SubApps(
			migrationapp.NewSubApp(),
			serverapp.NewSubApp(),
			workerapp.NewSubApp(),
		),
	)
}
