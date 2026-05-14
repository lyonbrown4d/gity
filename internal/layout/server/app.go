// Package server assembles the server dix application.
package server

import (
	"github.com/arcgolabs/dix"
	auditservice "github.com/lyonbrown4d/gity/internal/application/audit"
	issueservice "github.com/lyonbrown4d/gity/internal/application/issue"
	jobservice "github.com/lyonbrown4d/gity/internal/application/job"
	lfsservice "github.com/lyonbrown4d/gity/internal/application/lfs"
	mergerequestservice "github.com/lyonbrown4d/gity/internal/application/merge_request"
	organizationservice "github.com/lyonbrown4d/gity/internal/application/organization"
	packageregistryservice "github.com/lyonbrown4d/gity/internal/application/package_registry"
	pipelineservice "github.com/lyonbrown4d/gity/internal/application/pipeline"
	projectservice "github.com/lyonbrown4d/gity/internal/application/project"
	runnerservice "github.com/lyonbrown4d/gity/internal/application/runner"
	userservice "github.com/lyonbrown4d/gity/internal/application/user"
	wikiservice "github.com/lyonbrown4d/gity/internal/application/wiki"
	"github.com/lyonbrown4d/gity/internal/config"
	gitydebug "github.com/lyonbrown4d/gity/internal/debug"
	"github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	"github.com/lyonbrown4d/gity/internal/infrastructure/database"
	infraeventbus "github.com/lyonbrown4d/gity/internal/infrastructure/event_bus"
	"github.com/lyonbrown4d/gity/internal/infrastructure/git_exec"
	"github.com/lyonbrown4d/gity/internal/infrastructure/git_repo"
	"github.com/lyonbrown4d/gity/internal/infrastructure/git_transport"
	infralogger "github.com/lyonbrown4d/gity/internal/infrastructure/logger"
	inframapper "github.com/lyonbrown4d/gity/internal/infrastructure/mapperx"
	organizationrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/organization"
	organizationmemberrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/organization_member"
	projectrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project"
	projectauditeventrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_audit_event"
	projectbranchprotectionrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_branch_protection"
	projectcivariablerepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_ci_variable"
	projectissuerepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_issue"
	projectissueassigneerepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_issue_assignee"
	projectissueattachmentrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_issue_attachment"
	projectissuecommentrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_issue_comment"
	projectissuelabelrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_issue_label"
	projectjobrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_job"
	projectjobartifactrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_job_artifact"
	projectjoblogrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_job_log"
	projectlfslockrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_lfs_lock"
	projectlfsobjectrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_lfs_object"
	projectmemberrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_member"
	projectmergerequestrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_merge_request"
	projectmergerequestapprovalrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_merge_request_approval"
	projectmergerequestapprovalrulerepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_merge_request_approval_rule"
	projectmergerequestcommentrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_merge_request_comment"
	projectmergerequestparticipantrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_merge_request_participant"
	projectpackagerepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_package"
	projectpackagefilerepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_package_file"
	projectpackageversionrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_package_version"
	projectpipelinerepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_pipeline"
	projectpipelinejobrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_pipeline_job"
	projectrunnerrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_runner"
	projectwikipagerepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_wiki_page"
	userrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/user_token"
	searchindex "github.com/lyonbrown4d/gity/internal/infrastructure/search_index"
	infrastorage "github.com/lyonbrown4d/gity/internal/infrastructure/storage"
	auditendpoint "github.com/lyonbrown4d/gity/internal/interfaces/http/audit"
	authendpoint "github.com/lyonbrown4d/gity/internal/interfaces/http/auth"
	gittransportendpoint "github.com/lyonbrown4d/gity/internal/interfaces/http/git_transport"
	issueendpoint "github.com/lyonbrown4d/gity/internal/interfaces/http/issue"
	jobendpoint "github.com/lyonbrown4d/gity/internal/interfaces/http/job"
	lfsendpoint "github.com/lyonbrown4d/gity/internal/interfaces/http/lfs"
	mergerequestendpoint "github.com/lyonbrown4d/gity/internal/interfaces/http/merge_request"
	organizationendpoint "github.com/lyonbrown4d/gity/internal/interfaces/http/organization"
	packageregistryendpoint "github.com/lyonbrown4d/gity/internal/interfaces/http/package_registry"
	pipelineendpoint "github.com/lyonbrown4d/gity/internal/interfaces/http/pipeline"
	projectendpoint "github.com/lyonbrown4d/gity/internal/interfaces/http/project"
	runnerendpoint "github.com/lyonbrown4d/gity/internal/interfaces/http/runner"
	systemendpoint "github.com/lyonbrown4d/gity/internal/interfaces/http/system"
	userendpoint "github.com/lyonbrown4d/gity/internal/interfaces/http/user"
	wikiendpoint "github.com/lyonbrown4d/gity/internal/interfaces/http/wiki"
	httpapp "github.com/lyonbrown4d/gity/internal/interfaces/http_server"
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
		dix.LifecycleConcurrency(4),
		dix.RecentEvents(256),
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
			projectcivariablerepo.Module(),
			projectissuerepo.Module(),
			projectissueassigneerepo.Module(),
			projectissuecommentrepo.Module(),
			projectissueattachmentrepo.Module(),
			projectissuelabelrepo.Module(),
			projectjobartifactrepo.Module(),
			projectjoblogrepo.Module(),
			projectjobrepo.Module(),
			projectlfslockrepo.Module(),
			projectlfsobjectrepo.Module(),
			projectmemberrepo.Module(),
			projectmergerequestrepo.Module(),
			projectmergerequestcommentrepo.Module(),
			projectmergerequestapprovalrepo.Module(),
			projectmergerequestapprovalrulerepo.Module(),
			projectmergerequestparticipantrepo.Module(),
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
			searchindex.QueryModule(),
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
