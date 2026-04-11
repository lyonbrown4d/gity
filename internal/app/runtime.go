package app

import (
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
		dix.WithModules(serverModules()...),
	)
}

func NewWorkerApp() *dix.App {
	return dix.New(
		"gity-worker",
		dix.WithVersion("0.1.0"),
		dix.WithAppDescription("Gity background worker runtime on arcgo dix"),
		dix.WithModules(coreModules()...),
	)
}

func coreModules() []dix.Module {
	return []dix.Module{
		config.Module(),
		platformlogger.Module(),
		database.Module(),
		auth.Module(),
		gitexec.Module(),
		gittransport.Module(),
		coredb.Module(),
		namespacerepo.Module(),
		projectrepo.Module(),
		namespaceservice.Module(),
		projectservice.Module(),
	}
}

func serverModules() []dix.Module {
	modules := append([]dix.Module{}, coreModules()...)
	modules = append(
		modules,
		httpapp.Module(),
		systemendpoint.Module(),
		namespaceendpoint.Module(),
		projectendpoint.Module(),
	)
	return modules
}
