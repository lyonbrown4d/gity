package pipeline_test

import (
	"context"
	"log/slog"
	"strings"
	"testing"
	"time"

	jobservice "github.com/DaiYuANg/gity/internal/application/job"
	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
	"github.com/DaiYuANg/gity/internal/config"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_exec"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_repo"
	"github.com/DaiYuANg/gity/internal/infrastructure/persistence/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespace"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectjobrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_job"
	projectpipelinerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_pipeline"
	projectpipelinejobrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_pipeline_job"
	"github.com/DaiYuANg/gity/internal/testutil"
)

func TestProjectPipelineFlow(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	db := openTestDB(t)
	testutil.CleanupClose(t, "db", db)
	if schemaErr := core.EnsureSchema(ctx, db); schemaErr != nil {
		t.Fatalf("ensure schema: %v", schemaErr)
	}

	namespaceRepository, err := namespacerepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new namespace repo: %v", err)
	}
	projectRepository, err := projectrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new project repo: %v", err)
	}
	jobRepository, err := projectjobrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new job repo: %v", err)
	}
	pipelineRepository, err := projectpipelinerepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new pipeline repo: %v", err)
	}
	pipelineJobRepository, err := projectpipelinejobrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new pipeline job repo: %v", err)
	}
	jobSvc := jobservice.NewService(slog.Default(), projectRepository, jobRepository, nil, nil, nil)
	service := pipelineservice.NewService(projectRepository, pipelineRepository, pipelineJobRepository, jobSvc, jobRepository, nil)

	namespace, err := namespaceRepository.Create(ctx, namespacerepo.CreateInput{Kind: "group", Name: "Core Team", PathKey: "core-team"})
	if err != nil {
		t.Fatalf("create namespace: %v", err)
	}
	project, err := projectRepository.Create(ctx, projectrepo.CreateInput{NamespaceID: namespace.ID, Name: "Gity", PathKey: "gity", Visibility: "private"}, namespace)
	if err != nil {
		t.Fatalf("create project: %v", err)
	}

	created, err := service.CreatePipeline(ctx, project.ID, pipelineservice.CreatePipelineInput{
		Source:        "api",
		RefName:       "main",
		ConfigSource:  ".gity-ci.plano",
		ConfigContent: pipelineConfig(),
	})
	if err != nil {
		t.Fatalf("create pipeline: %v", err)
	}
	if created.Pipeline.IID != 1 || created.Pipeline.Name != "release" || created.Pipeline.Status != projectpipelinerepo.StatusPending {
		t.Fatalf("unexpected pipeline: %+v", created.Pipeline)
	}
	if len(created.Jobs) != 2 {
		t.Fatalf("jobs = %d", len(created.Jobs))
	}
	first := created.Jobs[0]
	if first.Status != projectjobrepo.StatusPending || first.ProjectJob.Kind != jobservice.KindScript {
		t.Fatalf("unexpected first job: %+v", first)
	}
	if !strings.Contains(first.ProjectJob.Payload, `"script":["go test ./..."]`) {
		t.Fatalf("unexpected first payload: %s", first.ProjectJob.Payload)
	}
	second := created.Jobs[1]
	if second.Status != "blocked" {
		t.Fatalf("expected second job to be blocked: %+v", second)
	}

	claimed, ok, err := jobSvc.ClaimProjectJob(ctx, project.ID, "runner-a", time.Minute)
	if err != nil {
		t.Fatalf("claim job: %v", err)
	}
	if !ok || claimed.ID != first.ProjectJob.ID {
		t.Fatalf("unexpected claimed job: ok=%v job=%+v", ok, claimed)
	}
	claimed, ok, err = jobSvc.ClaimProjectJob(ctx, project.ID, "runner-a", time.Minute)
	if err != nil {
		t.Fatalf("claim blocked job: %v", err)
	}
	if ok {
		t.Fatalf("blocked dependency job should not be claimable: %+v", claimed)
	}
	jobs, err := service.ListPipelineJobs(ctx, project.ID, created.Pipeline.ID)
	if err != nil {
		t.Fatalf("list pipeline jobs: %v", err)
	}
	if len(jobs) != 2 || jobs[0].Status != projectjobrepo.StatusRunning || jobs[1].Status != "blocked" {
		t.Fatalf("unexpected listed pipeline jobs: %+v", jobs)
	}
	if _, completeErr := jobSvc.CompleteProjectJob(ctx, project.ID, first.ProjectJob.ID, `{"ok":true}`); completeErr != nil {
		t.Fatalf("complete first job: %v", completeErr)
	}
	if refreshErr := service.RefreshProjectJob(ctx, project.ID, first.ProjectJob.ID); refreshErr != nil {
		t.Fatalf("refresh pipeline job: %v", refreshErr)
	}
	claimed, ok, err = jobSvc.ClaimProjectJob(ctx, project.ID, "runner-a", time.Minute)
	if err != nil {
		t.Fatalf("claim released job: %v", err)
	}
	if !ok || claimed.ID != second.ProjectJob.ID {
		t.Fatalf("dependency job was not released: ok=%v job=%+v", ok, claimed)
	}
}

func TestCreatePushPipelineFromRepositoryConfig(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	db := openTestDB(t)
	testutil.CleanupClose(t, "db", db)
	if schemaErr := core.EnsureSchema(ctx, db); schemaErr != nil {
		t.Fatalf("ensure schema: %v", schemaErr)
	}

	namespaceRepository, err := namespacerepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new namespace repo: %v", err)
	}
	projectRepository, err := projectrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new project repo: %v", err)
	}
	jobRepository, err := projectjobrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new job repo: %v", err)
	}
	pipelineRepository, err := projectpipelinerepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new pipeline repo: %v", err)
	}
	pipelineJobRepository, err := projectpipelinejobrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new pipeline job repo: %v", err)
	}

	repoRoot := t.TempDir()
	gitRunner := gitexec.NewRunner(config.Settings{Git: config.GitSettings{Bin: "git", RepoRoot: repoRoot}})
	gitRepository := gitrepo.NewService(config.Settings{Git: config.GitSettings{RepoRoot: repoRoot}})
	jobSvc := jobservice.NewService(slog.Default(), projectRepository, jobRepository, nil, nil, nil)
	service := pipelineservice.NewService(projectRepository, pipelineRepository, pipelineJobRepository, jobSvc, jobRepository, gitRepository)

	namespace, err := namespaceRepository.Create(ctx, namespacerepo.CreateInput{Kind: "group", Name: "Core Team", PathKey: "core-team"})
	if err != nil {
		t.Fatalf("create namespace: %v", err)
	}
	project, err := projectRepository.Create(ctx, projectrepo.CreateInput{NamespaceID: namespace.ID, Name: "Gity", PathKey: "gity", Visibility: "private", DefaultBranch: "main"}, namespace)
	if err != nil {
		t.Fatalf("create project: %v", err)
	}
	if initErr := gitRunner.InitBare(ctx, project.FullPath+".git", project.DefaultBranch); initErr != nil {
		t.Fatalf("init bare repo: %v", initErr)
	}
	if createCommitErr := gitRunner.CreateFileCommit(ctx, project.FullPath+".git", gitexec.CreateFileCommitInput{
		BranchName:  "main",
		FilePath:    ".gity-ci.plano",
		Content:     pipelineConfig(),
		Message:     "Add CI config",
		AuthorName:  "Gity Test",
		AuthorEmail: "test@gity.dev",
	}); createCommitErr != nil {
		t.Fatalf("create ci config commit: %v", createCommitErr)
	}
	branch, err := gitRepository.ListBranches(ctx, project.FullPath+".git", project.DefaultBranch)
	if err != nil {
		t.Fatalf("list branches: %v", err)
	}
	if len(branch) != 1 {
		t.Fatalf("unexpected branches: %+v", branch)
	}

	view, created, err := service.CreatePushPipeline(ctx, project.ID, "main", branch[0].Hash)
	if err != nil {
		t.Fatalf("create push pipeline: %v", err)
	}
	if !created || view.Pipeline.Source != "push" || view.Pipeline.RefName != "main" || view.Pipeline.CommitSHA != branch[0].Hash {
		t.Fatalf("unexpected push pipeline: created=%v view=%+v", created, view.Pipeline)
	}
	if len(view.Jobs) != 2 || !strings.Contains(view.Jobs[0].ProjectJob.Payload, `"project_full_path":"core-team/gity"`) {
		t.Fatalf("unexpected push pipeline jobs: %+v", view.Jobs)
	}

	duplicate, created, err := service.CreatePushPipeline(ctx, project.ID, "main", branch[0].Hash)
	if err != nil {
		t.Fatalf("deduplicate push pipeline: %v", err)
	}
	if created || duplicate.Pipeline.ID != view.Pipeline.ID {
		t.Fatalf("expected duplicate trigger to reuse pipeline: created=%v duplicate=%+v", created, duplicate.Pipeline)
	}
}
