package app

import (
	"github.com/DaiYuANg/gity/internal/config"
	authendpoint "github.com/DaiYuANg/gity/internal/endpoint/auth"
	gittransportendpoint "github.com/DaiYuANg/gity/internal/endpoint/gittransport"
	issueendpoint "github.com/DaiYuANg/gity/internal/endpoint/issue"
	jobendpoint "github.com/DaiYuANg/gity/internal/endpoint/job"
	lfsendpoint "github.com/DaiYuANg/gity/internal/endpoint/lfs"
	mergerequestendpoint "github.com/DaiYuANg/gity/internal/endpoint/mergerequest"
	namespaceendpoint "github.com/DaiYuANg/gity/internal/endpoint/namespace"
	packageregistryendpoint "github.com/DaiYuANg/gity/internal/endpoint/packageregistry"
	pipelineendpoint "github.com/DaiYuANg/gity/internal/endpoint/pipeline"
	projectendpoint "github.com/DaiYuANg/gity/internal/endpoint/project"
	runnerendpoint "github.com/DaiYuANg/gity/internal/endpoint/runner"
	systemendpoint "github.com/DaiYuANg/gity/internal/endpoint/system"
	userendpoint "github.com/DaiYuANg/gity/internal/endpoint/user"
	wikiendpoint "github.com/DaiYuANg/gity/internal/endpoint/wiki"
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
	projectbranchprotectionrepo "github.com/DaiYuANg/gity/internal/repository/projectbranchprotection"
	projectissuerepo "github.com/DaiYuANg/gity/internal/repository/projectissue"
	projectissueattachmentrepo "github.com/DaiYuANg/gity/internal/repository/projectissueattachment"
	projectissuecommentrepo "github.com/DaiYuANg/gity/internal/repository/projectissuecomment"
	projectjobrepo "github.com/DaiYuANg/gity/internal/repository/projectjob"
	projectjobartifactrepo "github.com/DaiYuANg/gity/internal/repository/projectjobartifact"
	projectjoblogrepo "github.com/DaiYuANg/gity/internal/repository/projectjoblog"
	projectlfslockrepo "github.com/DaiYuANg/gity/internal/repository/projectlfslock"
	projectlfsobjectrepo "github.com/DaiYuANg/gity/internal/repository/projectlfsobject"
	projectmergerequestrepo "github.com/DaiYuANg/gity/internal/repository/projectmergerequest"
	projectpackagerepo "github.com/DaiYuANg/gity/internal/repository/projectpackage"
	projectpackagefilerepo "github.com/DaiYuANg/gity/internal/repository/projectpackagefile"
	projectpackageversionrepo "github.com/DaiYuANg/gity/internal/repository/projectpackageversion"
	projectpipelinerepo "github.com/DaiYuANg/gity/internal/repository/projectpipeline"
	projectpipelinejobrepo "github.com/DaiYuANg/gity/internal/repository/projectpipelinejob"
	projectrunnerrepo "github.com/DaiYuANg/gity/internal/repository/projectrunner"
	projectwikipagerepo "github.com/DaiYuANg/gity/internal/repository/projectwikipage"
	userrepo "github.com/DaiYuANg/gity/internal/repository/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/repository/usertoken"
	issueservice "github.com/DaiYuANg/gity/internal/service/issue"
	jobservice "github.com/DaiYuANg/gity/internal/service/job"
	lfsservice "github.com/DaiYuANg/gity/internal/service/lfs"
	mergerequestservice "github.com/DaiYuANg/gity/internal/service/mergerequest"
	namespaceservice "github.com/DaiYuANg/gity/internal/service/namespace"
	packageregistryservice "github.com/DaiYuANg/gity/internal/service/packageregistry"
	pipelineservice "github.com/DaiYuANg/gity/internal/service/pipeline"
	projectservice "github.com/DaiYuANg/gity/internal/service/project"
	runnerservice "github.com/DaiYuANg/gity/internal/service/runner"
	userservice "github.com/DaiYuANg/gity/internal/service/user"
	wikiservice "github.com/DaiYuANg/gity/internal/service/wiki"
	jobrunner "github.com/DaiYuANg/gity/internal/worker/jobrunner"
	"github.com/arcgolabs/dix"
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
		projectbranchprotectionrepo.Module(),
		projectissuerepo.Module(),
		projectissuecommentrepo.Module(),
		projectissueattachmentrepo.Module(),
		projectjobartifactrepo.Module(),
		projectjoblogrepo.Module(),
		projectjobrepo.Module(),
		projectlfslockrepo.Module(),
		projectlfsobjectrepo.Module(),
		projectmergerequestrepo.Module(),
		projectpackagerepo.Module(),
		projectpackageversionrepo.Module(),
		projectpackagefilerepo.Module(),
		projectpipelinerepo.Module(),
		projectpipelinejobrepo.Module(),
		projectrunnerrepo.Module(),
		projectwikipagerepo.Module(),
		auth.Module(),
		gitexec.Module(),
		gitrepo.Module(),
		gittransport.Module(),
		platformstorage.Module(),
		userservice.Module(),
		namespaceservice.Module(),
		projectservice.Module(),
		issueservice.Module(),
		jobservice.Module(),
		lfsservice.Module(),
		mergerequestservice.Module(),
		packageregistryservice.Module(),
		pipelineservice.Module(),
		runnerservice.Module(),
		wikiservice.Module(),
	}
}

func serverAugmentModules() []dix.Module {
	return []dix.Module{
		httpapp.Module(),
		systemendpoint.Module(),
		authendpoint.Module(),
		gittransportendpoint.Module(),
		lfsendpoint.Module(),
		userendpoint.Module(),
		namespaceendpoint.Module(),
		projectendpoint.Module(),
		issueendpoint.Module(),
		jobendpoint.Module(),
		mergerequestendpoint.Module(),
		packageregistryendpoint.Module(),
		pipelineendpoint.Module(),
		runnerendpoint.Module(),
		wikiendpoint.Module(),
	}
}

func workerAugmentModules() []dix.Module {
	return []dix.Module{
		jobrunner.Module(),
	}
}
