package bootstrap

import (
	issueservice "github.com/DaiYuANg/gity/internal/application/issue"
	jobservice "github.com/DaiYuANg/gity/internal/application/job"
	lfsservice "github.com/DaiYuANg/gity/internal/application/lfs"
	mergerequestservice "github.com/DaiYuANg/gity/internal/application/mergerequest"
	namespaceservice "github.com/DaiYuANg/gity/internal/application/namespace"
	packageregistryservice "github.com/DaiYuANg/gity/internal/application/packageregistry"
	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	runnerservice "github.com/DaiYuANg/gity/internal/application/runner"
	userservice "github.com/DaiYuANg/gity/internal/application/user"
	wikiservice "github.com/DaiYuANg/gity/internal/application/wiki"
	"github.com/DaiYuANg/gity/internal/config"
	"github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/infrastructure/database"
	"github.com/DaiYuANg/gity/internal/infrastructure/gitexec"
	"github.com/DaiYuANg/gity/internal/infrastructure/gitrepo"
	"github.com/DaiYuANg/gity/internal/infrastructure/gittransport"
	infralogger "github.com/DaiYuANg/gity/internal/infrastructure/logger"
	inframapper "github.com/DaiYuANg/gity/internal/infrastructure/mapperx"
	coredb "github.com/DaiYuANg/gity/internal/infrastructure/persistence/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespace"
	namespacememberrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespacemember"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectbranchprotectionrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectbranchprotection"
	projectissuerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectissue"
	projectissueattachmentrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectissueattachment"
	projectissuecommentrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectissuecomment"
	projectjobrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectjob"
	projectjobartifactrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectjobartifact"
	projectjoblogrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectjoblog"
	projectlfslockrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectlfslock"
	projectlfsobjectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectlfsobject"
	projectmergerequestrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectmergerequest"
	projectpackagerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectpackage"
	projectpackagefilerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectpackagefile"
	projectpackageversionrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectpackageversion"
	projectpipelinerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectpipeline"
	projectpipelinejobrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectpipelinejob"
	projectrunnerrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectrunner"
	projectwikipagerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectwikipage"
	userrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/usertoken"
	infrastorage "github.com/DaiYuANg/gity/internal/infrastructure/storage"
	jobrunner "github.com/DaiYuANg/gity/internal/infrastructure/worker/jobrunner"
	authendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/auth"
	gittransportendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/gittransport"
	issueendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/issue"
	jobendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/job"
	lfsendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/lfs"
	mergerequestendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/mergerequest"
	namespaceendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/namespace"
	packageregistryendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/packageregistry"
	pipelineendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/pipeline"
	projectendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/project"
	runnerendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/runner"
	systemendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/system"
	userendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/user"
	wikiendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/wiki"
	httpapp "github.com/DaiYuANg/gity/internal/interfaces/httpserver"
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
		dix.UseLoggerErr1(infralogger.NewLogger),
		dix.WithModules(modules...),
	)
}

func migrationModules() []dix.Module {
	return []dix.Module{
		config.Module(),
		database.Module(),
		coredb.Module(),
	}
}

func sharedModules() []dix.Module {
	return []dix.Module{
		config.Module(),
		database.Module(),
		coredb.Module(),
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
		inframapper.Module(),
		infrastorage.Module(),
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
