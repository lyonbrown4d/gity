package bootstrap

import (
	issueservice "github.com/DaiYuANg/gity/internal/application/issue"
	jobservice "github.com/DaiYuANg/gity/internal/application/job"
	lfsservice "github.com/DaiYuANg/gity/internal/application/lfs"
	mergerequestservice "github.com/DaiYuANg/gity/internal/application/merge_request"
	namespaceservice "github.com/DaiYuANg/gity/internal/application/namespace"
	packageregistryservice "github.com/DaiYuANg/gity/internal/application/package_registry"
	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	runnerservice "github.com/DaiYuANg/gity/internal/application/runner"
	userservice "github.com/DaiYuANg/gity/internal/application/user"
	wikiservice "github.com/DaiYuANg/gity/internal/application/wiki"
	"github.com/DaiYuANg/gity/internal/config"
	"github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/infrastructure/database"
	infraeventbus "github.com/DaiYuANg/gity/internal/infrastructure/event_bus"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_exec"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_repo"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_transport"
	infralogger "github.com/DaiYuANg/gity/internal/infrastructure/logger"
	inframapper "github.com/DaiYuANg/gity/internal/infrastructure/mapperx"
	coredb "github.com/DaiYuANg/gity/internal/infrastructure/persistence/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespace"
	namespacememberrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespace_member"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectbranchprotectionrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_branch_protection"
	projectissuerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_issue"
	projectissueattachmentrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_issue_attachment"
	projectissuecommentrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_issue_comment"
	projectjobrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_job"
	projectjobartifactrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_job_artifact"
	projectjoblogrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_job_log"
	projectlfslockrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_lfs_lock"
	projectlfsobjectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_lfs_object"
	projectmergerequestrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_merge_request"
	projectpackagerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_package"
	projectpackagefilerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_package_file"
	projectpackageversionrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_package_version"
	projectpipelinerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_pipeline"
	projectpipelinejobrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_pipeline_job"
	projectrunnerrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_runner"
	projectwikipagerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_wiki_page"
	userrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user_token"
	infrastorage "github.com/DaiYuANg/gity/internal/infrastructure/storage"
	jobrunner "github.com/DaiYuANg/gity/internal/infrastructure/worker/job_runner"
	authendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/auth"
	gittransportendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/git_transport"
	issueendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/issue"
	jobendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/job"
	lfsendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/lfs"
	mergerequestendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/merge_request"
	namespaceendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/namespace"
	packageregistryendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/package_registry"
	pipelineendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/pipeline"
	projectendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/project"
	runnerendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/runner"
	systemendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/system"
	userendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/user"
	wikiendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/wiki"
	httpapp "github.com/DaiYuANg/gity/internal/interfaces/http_server"
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

func newApp(name, description string, modules []dix.Module) *dix.App {
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
		infraeventbus.Module(),
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
