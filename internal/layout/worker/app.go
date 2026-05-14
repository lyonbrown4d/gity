// Package worker assembles the worker dix application.
package worker

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
	jobrunner "github.com/lyonbrown4d/gity/internal/infrastructure/worker/job_runner"
)

// NewApp builds the standalone background worker application.
func NewApp() *dix.App {
	return dix.New(
		"gity-worker",
		appOptions(
			"cmd.worker.meta",
			"gity-worker",
			"Gity background worker runtime",
			sharedRuntimeModule(),
			workerRuntimeModule(),
		)...,
	)
}

// NewSubApp builds the worker sub-application used by standalone.
func NewSubApp() *dix.App {
	return dix.NewSubApp(
		"worker",
		appOptions(
			"cmd.worker.meta",
			"worker",
			"Gity background worker subapp runtime",
			sharedRuntimeModule(),
			workerRuntimeModule(),
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
		"cmd.worker.shared",
		dix.Description("Worker shared runtime composition"),
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
		"cmd.worker.repositories",
		dix.Description("Worker repository adapters"),
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
		"cmd.worker.infrastructure",
		dix.Description("Worker infrastructure adapters"),
		dix.Imports(
			auth.Module(),
			gitexec.Module(),
			gitrepo.Module(),
			gittransport.Module(),
			inframapper.Module(),
			infrastorage.Module(),
			infraeventbus.Module(),
			searchindex.Module(),
		),
	)
}

func applicationRuntimeModule() dix.Module {
	return dix.NewModule(
		"cmd.worker.application",
		dix.Description("Worker application services"),
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

func workerRuntimeModule() dix.Module {
	return dix.NewModule(
		"cmd.worker.jobs",
		dix.Description("Worker command composition"),
		dix.Imports(
			jobrunner.Module(),
		),
	)
}
