package app

import (
	"github.com/DaiYuANg/arcgo/dix"
	"github.com/DaiYuANg/gity/internal/config"
	gittransportendpoint "github.com/DaiYuANg/gity/internal/endpoint/gittransport"
	issueendpoint "github.com/DaiYuANg/gity/internal/endpoint/issue"
	lfsendpoint "github.com/DaiYuANg/gity/internal/endpoint/lfs"
	mergerequestendpoint "github.com/DaiYuANg/gity/internal/endpoint/mergerequest"
	namespaceendpoint "github.com/DaiYuANg/gity/internal/endpoint/namespace"
	packageregistryendpoint "github.com/DaiYuANg/gity/internal/endpoint/packageregistry"
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
	platformstorage "github.com/DaiYuANg/gity/internal/platform/storage"
	namespacerepo "github.com/DaiYuANg/gity/internal/repository/namespace"
	namespacememberrepo "github.com/DaiYuANg/gity/internal/repository/namespacemember"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	projectissuerepo "github.com/DaiYuANg/gity/internal/repository/projectissue"
	projectissueattachmentrepo "github.com/DaiYuANg/gity/internal/repository/projectissueattachment"
	projectissuecommentrepo "github.com/DaiYuANg/gity/internal/repository/projectissuecomment"
	projectlfslockrepo "github.com/DaiYuANg/gity/internal/repository/projectlfslock"
	projectlfsobjectrepo "github.com/DaiYuANg/gity/internal/repository/projectlfsobject"
	projectmergerequestrepo "github.com/DaiYuANg/gity/internal/repository/projectmergerequest"
	projectpackagerepo "github.com/DaiYuANg/gity/internal/repository/projectpackage"
	projectpackagefilerepo "github.com/DaiYuANg/gity/internal/repository/projectpackagefile"
	projectpackageversionrepo "github.com/DaiYuANg/gity/internal/repository/projectpackageversion"
	userrepo "github.com/DaiYuANg/gity/internal/repository/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/repository/usertoken"
	issueservice "github.com/DaiYuANg/gity/internal/service/issue"
	lfsservice "github.com/DaiYuANg/gity/internal/service/lfs"
	mergerequestservice "github.com/DaiYuANg/gity/internal/service/mergerequest"
	namespaceservice "github.com/DaiYuANg/gity/internal/service/namespace"
	packageregistryservice "github.com/DaiYuANg/gity/internal/service/packageregistry"
	projectservice "github.com/DaiYuANg/gity/internal/service/project"
	userservice "github.com/DaiYuANg/gity/internal/service/user"
)

func NewMigrationApp() *dix.App {
	return newApp(
		"gity-migration",
		"Gity database migration runtime",
		migrationModules(),
	)
}

func NewServerApp() *dix.App {
	return newApp(
		"gity-server",
		"Gity HTTP server runtime",
		append(sharedModules(), serverAugmentModules()...),
	)
}

func NewWorkerApp() *dix.App {
	return newApp(
		"gity-worker",
		"Gity background worker runtime",
		append(sharedModules(), workerAugmentModules()...),
	)
}

func NewStandaloneApp() *dix.App {
	modules := append(sharedModules(), serverAugmentModules()...)
	modules = append(modules, workerAugmentModules()...)
	return newApp(
		"gity-standalone",
		"Gity standalone runtime with server and worker components",
		modules,
	)
}

func newApp(name string, description string, modules []dix.Module) *dix.App {
	return dix.New(
		name,
		dix.WithVersion("0.1.0"),
		dix.WithAppDescription(description),
		dix.UseLoggerErr1(platformlogger.NewLogger),
		dix.WithModules(modules...),
	)
}

func migrationModules() []dix.Module {
	return []dix.Module{
		config.Module(),
		database.Module(),
	}
}

func sharedModules() []dix.Module {
	return []dix.Module{
		config.Module(),
		database.Module(),
		userrepo.Module(),
		usertokenrepo.Module(),
		namespacerepo.Module(),
		namespacememberrepo.Module(),
		projectrepo.Module(),
		projectissuerepo.Module(),
		projectissuecommentrepo.Module(),
		projectissueattachmentrepo.Module(),
		projectlfslockrepo.Module(),
		projectlfsobjectrepo.Module(),
		projectmergerequestrepo.Module(),
		projectpackagerepo.Module(),
		projectpackageversionrepo.Module(),
		projectpackagefilerepo.Module(),
		auth.Module(),
		gitexec.Module(),
		gitrepo.Module(),
		gittransport.Module(),
		platformstorage.Module(),
		userservice.Module(),
		namespaceservice.Module(),
		projectservice.Module(),
		issueservice.Module(),
		lfsservice.Module(),
		mergerequestservice.Module(),
		packageregistryservice.Module(),
	}
}

func serverAugmentModules() []dix.Module {
	return []dix.Module{
		httpapp.Module(),
		systemendpoint.Module(),
		gittransportendpoint.Module(),
		lfsendpoint.Module(),
		userendpoint.Module(),
		namespaceendpoint.Module(),
		projectendpoint.Module(),
		issueendpoint.Module(),
		mergerequestendpoint.Module(),
		packageregistryendpoint.Module(),
	}
}

func workerAugmentModules() []dix.Module {
	return []dix.Module{}
}
