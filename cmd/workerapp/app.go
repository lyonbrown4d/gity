// Package workerapp assembles the worker dix application.
package workerapp

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
	jobrunner "github.com/DaiYuANg/gity/internal/infrastructure/worker/job_runner"
	"github.com/arcgolabs/dix"
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
