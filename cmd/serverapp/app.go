// Package serverapp assembles the server dix application.
package serverapp

import (
	auditservice "github.com/DaiYuANg/gity/internal/application/audit"
	issueservice "github.com/DaiYuANg/gity/internal/application/issue"
	jobservice "github.com/DaiYuANg/gity/internal/application/job"
	lfsservice "github.com/DaiYuANg/gity/internal/application/lfs"
	mergerequestservice "github.com/DaiYuANg/gity/internal/application/merge_request"
	organizationservice "github.com/DaiYuANg/gity/internal/application/organization"
	packageregistryservice "github.com/DaiYuANg/gity/internal/application/package_registry"
	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	runnerservice "github.com/DaiYuANg/gity/internal/application/runner"
	userservice "github.com/DaiYuANg/gity/internal/application/user"
	wikiservice "github.com/DaiYuANg/gity/internal/application/wiki"
	"github.com/DaiYuANg/gity/internal/config"
	gitydebug "github.com/DaiYuANg/gity/internal/debug"
	"github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/infrastructure/database"
	infraeventbus "github.com/DaiYuANg/gity/internal/infrastructure/event_bus"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_exec"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_repo"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_transport"
	infralogger "github.com/DaiYuANg/gity/internal/infrastructure/logger"
	inframapper "github.com/DaiYuANg/gity/internal/infrastructure/mapperx"
	organizationrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/organization"
	organizationmemberrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/organization_member"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectauditeventrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_audit_event"
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
	auditendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/audit"
	authendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/auth"
	gittransportendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/git_transport"
	issueendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/issue"
	jobendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/job"
	lfsendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/lfs"
	mergerequestendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/merge_request"
	organizationendpoint "github.com/DaiYuANg/gity/internal/interfaces/http/organization"
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

// NewApp builds the standalone HTTP server application.
func NewApp() *dix.App {
	return dix.New(
		"gity-server",
		appOptions(
			"cmd.server.meta",
			"gity-server",
			"Gity HTTP server runtime",
			sharedRuntimeModule(),
			serverRuntimeModule(),
		)...,
	)
}

// NewSubApp builds the server sub-application used by standalone.
func NewSubApp() *dix.App {
	return dix.NewSubApp(
		"server",
		appOptions(
			"cmd.server.meta",
			"server",
			"Gity HTTP server subapp runtime",
			sharedRuntimeModule(),
			serverRuntimeModule(),
		)...,
	)
}

func appOptions(metaModuleName, appName, description string, modules ...dix.Module) []dix.AppOption {
	runtimeModules := make([]dix.Module, 0, len(modules)+1)
	runtimeModules = append(runtimeModules, gitydebug.Module(metaModuleName, appName, description))
	runtimeModules = append(runtimeModules, modules...)
	return []dix.AppOption{
		dix.UseLoggerErr1(infralogger.NewLogger),
		dix.Modules(runtimeModules...),
	}
}

func sharedRuntimeModule() dix.Module {
	return dix.NewModule(
		"cmd.server.shared",
		dix.Description("Server shared runtime composition"),
		dix.Imports(
			config.Module(),
			database.Module(),
			repositoryRuntimeModule(),
			infrastructureRuntimeModule(),
			applicationRuntimeModule(),
		),
	)
}

func repositoryRuntimeModule() dix.Module {
	return dix.NewModule(
		"cmd.server.repositories",
		dix.Description("Server repository adapters"),
		dix.Imports(
			userrepo.Module(),
			usertokenrepo.Module(),
			organizationrepo.Module(),
			organizationmemberrepo.Module(),
			projectrepo.Module(),
			projectauditeventrepo.Module(),
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
		),
	)
}

func infrastructureRuntimeModule() dix.Module {
	return dix.NewModule(
		"cmd.server.infrastructure",
		dix.Description("Server infrastructure adapters"),
		dix.Imports(
			auth.Module(),
			gitexec.Module(),
			gitrepo.Module(),
			gittransport.Module(),
			inframapper.Module(),
			infrastorage.Module(),
			infraeventbus.Module(),
		),
	)
}

func applicationRuntimeModule() dix.Module {
	return dix.NewModule(
		"cmd.server.application",
		dix.Description("Server application services"),
		dix.Imports(
			userservice.Module(),
			auditservice.Module(),
			organizationservice.Module(),
			projectservice.Module(),
			issueservice.Module(),
			jobservice.Module(),
			lfsservice.Module(),
			mergerequestservice.Module(),
			packageregistryservice.Module(),
			pipelineservice.Module(),
			runnerservice.Module(),
			wikiservice.Module(),
		),
	)
}

func serverRuntimeModule() dix.Module {
	return dix.NewModule(
		"cmd.server.http",
		dix.Description("HTTP server command composition"),
		dix.Imports(
			httpapp.Module(),
			endpointRuntimeModule(),
		),
	)
}

func endpointRuntimeModule() dix.Module {
	return dix.NewModule(
		"cmd.server.http.endpoints",
		dix.Description("HTTP endpoints"),
		dix.Imports(
			systemendpoint.Module(),
			authendpoint.Module(),
			auditendpoint.Module(),
			gittransportendpoint.Module(),
			lfsendpoint.Module(),
			userendpoint.Module(),
			organizationendpoint.Module(),
			projectendpoint.Module(),
			issueendpoint.Module(),
			jobendpoint.Module(),
			mergerequestendpoint.Module(),
			packageregistryendpoint.Module(),
			pipelineendpoint.Module(),
			runnerendpoint.Module(),
			wikiendpoint.Module(),
		),
	)
}
