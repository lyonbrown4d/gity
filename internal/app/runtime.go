package app

import (
	"github.com/DaiYuANg/arcgo/dix"
	"github.com/DaiYuANg/gity/internal/config"
	gittransportendpoint "github.com/DaiYuANg/gity/internal/endpoint/gittransport"
	namespaceendpoint "github.com/DaiYuANg/gity/internal/endpoint/namespace"
	projectendpoint "github.com/DaiYuANg/gity/internal/endpoint/project"
	systemendpoint "github.com/DaiYuANg/gity/internal/endpoint/system"
	userendpoint "github.com/DaiYuANg/gity/internal/endpoint/user"
	httpapp "github.com/DaiYuANg/gity/internal/http"
	"github.com/DaiYuANg/gity/internal/platform/auth"
	"github.com/DaiYuANg/gity/internal/platform/database"
	"github.com/DaiYuANg/gity/internal/platform/gitexec"
	"github.com/DaiYuANg/gity/internal/platform/gitrepo"
	"github.com/DaiYuANg/gity/internal/platform/gittransport"
	platformlogger "github.com/DaiYuANg/gity/internal/platform/logger"
	coredb "github.com/DaiYuANg/gity/internal/repository/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/repository/namespace"
	namespacememberrepo "github.com/DaiYuANg/gity/internal/repository/namespacemember"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	userrepo "github.com/DaiYuANg/gity/internal/repository/user"
	namespaceservice "github.com/DaiYuANg/gity/internal/service/namespace"
	projectservice "github.com/DaiYuANg/gity/internal/service/project"
	userservice "github.com/DaiYuANg/gity/internal/service/user"
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
		gitrepo.Module(),
		gittransport.Module(),
		coredb.Module(),
		userrepo.Module(),
		namespacerepo.Module(),
		namespacememberrepo.Module(),
		projectrepo.Module(),
		userservice.Module(),
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
		gittransportendpoint.Module(),
		userendpoint.Module(),
		namespaceendpoint.Module(),
		projectendpoint.Module(),
	)
	return modules
}
