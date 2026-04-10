package app

import (
	"context"

	"github.com/DaiYuANg/arcgo/dbx"
	"github.com/DaiYuANg/arcgo/dix"
	"github.com/DaiYuANg/gity/internal/config"
	namespaceendpoint "github.com/DaiYuANg/gity/internal/endpoint/namespace"
	projectendpoint "github.com/DaiYuANg/gity/internal/endpoint/project"
	systemendpoint "github.com/DaiYuANg/gity/internal/endpoint/system"
	httpapp "github.com/DaiYuANg/gity/internal/http"
	"github.com/DaiYuANg/gity/internal/platform/auth"
	"github.com/DaiYuANg/gity/internal/platform/database"
	"github.com/DaiYuANg/gity/internal/platform/gitexec"
	"github.com/DaiYuANg/gity/internal/platform/gittransport"
	platformlogger "github.com/DaiYuANg/gity/internal/platform/logger"
	coredb "github.com/DaiYuANg/gity/internal/repository/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/repository/namespace"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	namespaceservice "github.com/DaiYuANg/gity/internal/service/namespace"
	projectservice "github.com/DaiYuANg/gity/internal/service/project"
)

func NewServerApp() *dix.App {
	return dix.New(
		"gity-server",
		dix.WithVersion("0.1.0"),
		dix.WithAppDescription("Gity backend rewrite on arcgo dix + dbx + httpx + authx"),
		dix.WithModules(coreModule(), httpModule()),
	)
}

func NewWorkerApp() *dix.App {
	return dix.New(
		"gity-worker",
		dix.WithVersion("0.1.0"),
		dix.WithAppDescription("Gity background worker runtime on arcgo dix"),
		dix.WithModules(coreModule()),
	)
}

func coreModule() dix.Module {
	return dix.NewModule(
		"core",
		dix.Description("Shared runtime services"),
		dix.Providers(
			dix.ProviderErr0(config.NewConfig),
			dix.ProviderErr1(config.NewSettings),
			dix.ProviderErr1(platformlogger.NewLogger),
			dix.ProviderErr2(database.NewDatabase),
			dix.Provider0(auth.NewRuntime),
			dix.Provider1(gitexec.NewRunner),
			dix.Provider1(gittransport.NewService),
			dix.ProviderErr1(namespacerepo.NewRepository),
			dix.ProviderErr1(projectrepo.NewRepository),
			dix.Provider2(namespaceservice.NewService),
			dix.Provider4(projectservice.NewService),
		),
		dix.Hooks(
			dix.OnStart(func(ctx context.Context, db *dbx.DB) error {
				return coredb.EnsureSchema(ctx, db)
			}),
		),
	)
}

func httpModule() dix.Module {
	return dix.NewModule(
		"http",
		dix.Description("HTTP server, routes, and lifecycle"),
		dix.Providers(
			dix.ProviderErr2(httpapp.NewServer),
			dix.Provider3(httpapp.NewHost),
		),
		dix.Invokes(
			dix.Invoke2(systemendpoint.RegisterRoutes),
			dix.Invoke2(namespaceendpoint.RegisterRoutes),
			dix.Invoke2(projectendpoint.RegisterRoutes),
		),
		dix.Hooks(
			dix.OnStart(func(ctx context.Context, host *httpapp.Host) error {
				return host.Start(ctx)
			}),
			dix.OnStop(func(ctx context.Context, host *httpapp.Host) error {
				return host.Stop(ctx)
			}),
		),
	)
}
