// Package searchindex assembles search index maintenance command runtime.
package searchindex

import (
	"context"

	"github.com/arcgolabs/dix"
	"github.com/lyonbrown4d/gity/internal/application/ports"
	"github.com/lyonbrown4d/gity/internal/config"
	gitydebug "github.com/lyonbrown4d/gity/internal/debug"
	"github.com/lyonbrown4d/gity/internal/infrastructure/database"
	infralogger "github.com/lyonbrown4d/gity/internal/infrastructure/logger"
	projectrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project"
	searchindexruntime "github.com/lyonbrown4d/gity/internal/infrastructure/search_index"
	"github.com/samber/oops"
)

// NewApp builds the standalone search index rebuild application.
func NewApp() *dix.App {
	return dix.New(
		"gity-search-index",
		appOptions(
			"cmd.search_index.meta",
			"gity-search-index",
			"Gity search index rebuild runtime",
			searchIndexRuntimeModule(),
		)...,
	)
}

func appOptions(metaModuleName, appName, description string, modules ...dix.Module) []dix.AppOption {
	runtimeModules := make([]dix.Module, 0, len(modules)+1)
	runtimeModules = append(runtimeModules, gitydebug.Module(metaModuleName, appName, description))
	runtimeModules = append(runtimeModules, modules...)
	return []dix.AppOption{
		dix.UseLoggerErr1(infralogger.NewLogger),
		dix.LifecycleConcurrency(2),
		dix.RecentEvents(128),
		dix.Modules(runtimeModules...),
	}
}

func searchIndexRuntimeModule() dix.Module {
	return dix.NewModule(
		"cmd.search_index.runtime",
		dix.Description("Search index rebuild runtime composition"),
		dix.Imports(
			config.Module(),
			database.Module(),
			projectrepo.Module(),
			searchindexruntime.QueryModule(),
		),
	)
}

// Run starts the search index rebuild application and rebuilds all or selected project indexes.
func Run(ctx context.Context, projectIDs ...int64) (err error) {
	if ctx == nil {
		return oops.In("cmd.search_index").New("search index context is required")
	}

	app := NewApp()
	runtime, err := app.Start(ctx)
	if err != nil {
		return oops.In("cmd.search_index").Wrapf(err, "start search index app")
	}

	defer func() {
		if stopErr := runtime.Stop(context.WithoutCancel(ctx)); stopErr != nil && err == nil {
			err = oops.In("cmd.search_index").Wrapf(stopErr, "stop search index app")
		}
	}()

	searchService, err := dix.ResolveAs[*searchindexruntime.Service](runtime.Container())
	if err != nil {
		return oops.In("cmd.search_index").Wrapf(err, "resolve search index service")
	}
	projectRepo, err := dix.ResolveAs[ports.ProjectRepository](runtime.Container())
	if err != nil {
		return oops.In("cmd.search_index").Wrapf(err, "resolve project repository")
	}

	if len(projectIDs) == 0 {
		return oops.In("cmd.search_index").Wrapf(searchService.RefreshAll(ctx), "refresh all project search indexes")
	}

	return oops.In("cmd.search_index").Wrapf(refreshSelectedProjects(ctx, projectRepo, searchService, projectIDs), "refresh selected project search indexes")
}

func refreshSelectedProjects(
	ctx context.Context,
	projectRepo ports.ProjectRepository,
	searchService *searchindexruntime.Service,
	projectIDs []int64,
) error {
	for _, projectID := range projectIDs {
		project, err := projectRepo.GetByID(ctx, projectID)
		if err != nil {
			return oops.In("cmd.search_index").With("project_id", projectID).Wrapf(err, "load project")
		}
		if err := searchService.RefreshProject(ctx, project); err != nil {
			return oops.In("cmd.search_index").With("project_id", projectID).Wrapf(err, "refresh project search index")
		}
	}

	return nil
}
