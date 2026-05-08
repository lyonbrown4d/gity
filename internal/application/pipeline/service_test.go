package pipeline_test

import (
	"context"
	"fmt"
	"log/slog"
	"path/filepath"
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

	"github.com/arcgolabs/dbx"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
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

func TestProjectPipelineFailureCancelsPendingJobs(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	env := setupPipelineTest(t, ctx)
	created, err := env.Service.CreatePipeline(ctx, env.ProjectID, pipelineservice.CreatePipelineInput{
		Source:        "api",
		RefName:       "main",
		ConfigSource:  ".gity-ci.plano",
		ConfigContent: parallelPipelineConfig(),
	})
	if err != nil {
		t.Fatalf("create pipeline: %v", err)
	}
	claimed, ok, err := env.JobService.ClaimProjectJob(ctx, env.ProjectID, "runner-a", time.Minute)
	if err != nil {
		t.Fatalf("claim first job: %v", err)
	}
	if !ok {
		t.Fatalf("expected first job to be claimed")
	}
	if _, failErr := env.JobService.FailProjectJob(ctx, env.ProjectID, claimed.ID, "failed", time.Second); failErr != nil {
		t.Fatalf("fail first job: %v", failErr)
	}
	if refreshErr := env.Service.RefreshProjectJob(ctx, env.ProjectID, claimed.ID); refreshErr != nil {
		t.Fatalf("refresh failed job: %v", refreshErr)
	}
	view, err := env.Service.GetPipeline(ctx, env.ProjectID, created.Pipeline.ID)
	if err != nil {
		t.Fatalf("get pipeline: %v", err)
	}
	if view.Pipeline.Status != projectpipelinerepo.StatusFailed {
		t.Fatalf("pipeline status = %s", view.Pipeline.Status)
	}
	canceled := 0
	failed := 0
	for _, item := range view.Jobs {
		switch item.Status {
		case projectjobrepo.StatusFailed:
			failed++
		case projectjobrepo.StatusCancelled:
			canceled++
		}
	}
	if failed != 1 || canceled != 1 {
		t.Fatalf("expected one failed and one canceled job: %+v", view.Jobs)
	}
}

func TestCancelPipelineCancelsPendingJobs(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	env := setupPipelineTest(t, ctx)
	created, err := env.Service.CreatePipeline(ctx, env.ProjectID, pipelineservice.CreatePipelineInput{
		Source:        "api",
		RefName:       "main",
		ConfigSource:  ".gity-ci.plano",
		ConfigContent: pipelineConfig(),
	})
	if err != nil {
		t.Fatalf("create pipeline: %v", err)
	}
	view, err := env.Service.CancelPipeline(ctx, env.ProjectID, created.Pipeline.ID)
	if err != nil {
		t.Fatalf("cancel pipeline: %v", err)
	}
	if view.Pipeline.Status != projectpipelinerepo.StatusCancelled {
		t.Fatalf("pipeline status = %s", view.Pipeline.Status)
	}
	for _, item := range view.Jobs {
		if item.Status != projectjobrepo.StatusCancelled {
			t.Fatalf("expected canceled job: %+v", item)
		}
	}
}

func TestRetryPipelineResetsJobs(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	env := setupPipelineTest(t, ctx)
	created, err := env.Service.CreatePipeline(ctx, env.ProjectID, pipelineservice.CreatePipelineInput{
		Source:        "api",
		RefName:       "main",
		ConfigSource:  ".gity-ci.plano",
		ConfigContent: pipelineConfig(),
	})
	if err != nil {
		t.Fatalf("create pipeline: %v", err)
	}
	claimed, ok, err := env.JobService.ClaimProjectJob(ctx, env.ProjectID, "runner-a", time.Minute)
	if err != nil {
		t.Fatalf("claim first job: %v", err)
	}
	if !ok {
		t.Fatalf("expected first job to be claimed")
	}
	if _, failErr := env.JobService.FailProjectJob(ctx, env.ProjectID, claimed.ID, "failed", time.Second); failErr != nil {
		t.Fatalf("fail first job: %v", failErr)
	}
	if refreshErr := env.Service.RefreshProjectJob(ctx, env.ProjectID, claimed.ID); refreshErr != nil {
		t.Fatalf("refresh failed job: %v", refreshErr)
	}
	retried, err := env.Service.RetryPipeline(ctx, env.ProjectID, created.Pipeline.ID)
	if err != nil {
		t.Fatalf("retry pipeline: %v", err)
	}
	if retried.Pipeline.Status != projectpipelinerepo.StatusPending {
		t.Fatalf("pipeline status = %s", retried.Pipeline.Status)
	}
	jobs, err := env.JobService.ListProjectJobs(ctx, env.ProjectID)
	if err != nil {
		t.Fatalf("list jobs: %v", err)
	}
	for _, item := range jobs {
		if item.Status != projectjobrepo.StatusPending {
			t.Fatalf("expected pending job status after retry: %+v", item)
		}
		if item.Attempts != 0 {
			t.Fatalf("attempts should reset on retry: %+v", item)
		}
	}
	reclaimed, ok, err := env.JobService.ClaimProjectJob(ctx, env.ProjectID, "runner-a", time.Minute)
	if err != nil {
		t.Fatalf("claim retried job: %v", err)
	}
	if !ok || reclaimed.ID != claimed.ID {
		t.Fatalf("expected retried first job to be claimable: %+v", reclaimed)
	}
}

func pipelineConfig() string {
	return `
pipeline {
  name = "release"
}

stage test {
  timeout_seconds = 300
  run {
    shell("go test ./...")
  }
}

stage build {
  needs = [test]
  artifacts = ["dist/**"]
  run {
    exec("go", "build", "./cmd/server")
  }
}
`
}

func parallelPipelineConfig() string {
	return `
pipeline {
  name = "parallel"
}

stage test {
  run {
    shell("go test ./...")
  }
}

stage lint {
  run {
    shell("golangci-lint run ./...")
  }
}
`
}

type pipelineTestEnv struct {
	ProjectID  int64
	JobService *jobservice.Service
	Service    *pipelineservice.Service
}

func setupPipelineTest(t *testing.T, ctx context.Context) pipelineTestEnv {
	t.Helper()
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
	return pipelineTestEnv{ProjectID: project.ID, JobService: jobSvc, Service: service}
}

func openTestDB(t *testing.T) *dbx.DB {
	t.Helper()
	dbPath := filepath.Join(t.TempDir(), "gity-pipeline-test.db")
	db, err := dbx.Open(
		dbx.WithDriver("sqlite"),
		dbx.WithDSN(fmt.Sprintf("file:%s?_pragma=foreign_keys(1)", dbPath)),
		dbx.WithDialect(sqliteDialect.New()),
	)
	if err != nil {
		t.Fatalf("open db: %v", err)
	}
	return db
}
