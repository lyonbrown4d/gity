package pipeline_test

import (
	"context"
	"strings"
	"testing"
	"time"

	jobservice "github.com/lyonbrown4d/gity/internal/application/job"
	pipelineservice "github.com/lyonbrown4d/gity/internal/application/pipeline"
	"github.com/lyonbrown4d/gity/internal/infrastructure/git_exec"
	projectjobrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_job"
	projectpipelinerepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_pipeline"
	"github.com/lyonbrown4d/gity/internal/testutil"
)

func TestProjectPipelineFlow(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	env := setupPipelineTest(ctx, t)
	created := createTestPipeline(ctx, t, env, pipelineConfig())
	assertCreatedPipeline(t, created)
	assertPipelineJobRelease(ctx, t, env, created)
}

func assertCreatedPipeline(t *testing.T, created pipelineservice.PipelineView) {
	t.Helper()

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
}

func assertPipelineJobRelease(ctx context.Context, t *testing.T, env pipelineTestEnv, created pipelineservice.PipelineView) {
	t.Helper()

	first := created.Jobs[0]
	second := created.Jobs[1]
	claimed, ok, err := env.JobService.ClaimProjectJob(ctx, env.ProjectID, "runner-a", time.Minute)
	testutil.RequireNoError(t, err, "claim job")
	if !ok || claimed.ID != first.ProjectJob.ID {
		t.Fatalf("unexpected claimed job: ok=%v job=%+v", ok, claimed)
	}
	claimed, ok, err = env.JobService.ClaimProjectJob(ctx, env.ProjectID, "runner-a", time.Minute)
	testutil.RequireNoError(t, err, "claim blocked job")
	if ok {
		t.Fatalf("blocked dependency job should not be claimable: %+v", claimed)
	}
	jobs := testutil.Must(env.Service.ListPipelineJobs(ctx, env.ProjectID, created.Pipeline.ID))
	if len(jobs) != 2 || jobs[0].Status != projectjobrepo.StatusRunning || jobs[1].Status != "blocked" {
		t.Fatalf("unexpected listed pipeline jobs: %+v", jobs)
	}
	_, completeErr := env.JobService.CompleteProjectJob(ctx, env.ProjectID, first.ProjectJob.ID, `{"ok":true}`)
	testutil.RequireNoError(t, completeErr, "complete first job")
	testutil.RequireNoError(t, env.Service.RefreshProjectJob(ctx, env.ProjectID, first.ProjectJob.ID), "refresh pipeline job")
	claimed, ok, err = env.JobService.ClaimProjectJob(ctx, env.ProjectID, "runner-a", time.Minute)
	testutil.RequireNoError(t, err, "claim released job")
	if !ok || claimed.ID != second.ProjectJob.ID {
		t.Fatalf("dependency job was not released: ok=%v job=%+v", ok, claimed)
	}
}

func TestCreatePushPipelineFromRepositoryConfig(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	env := setupGitPipelineTest(ctx, t)
	seedPipelineConfigCommit(ctx, t, env)
	branch := testutil.Must(env.GitRepository.ListBranches(ctx, env.ProjectFullPath+".git", env.DefaultBranch))
	if len(branch) != 1 {
		t.Fatalf("unexpected branches: %+v", branch)
	}

	view, created, err := env.Service.CreatePushPipeline(ctx, env.ProjectID, "main", branch[0].Hash)
	testutil.RequireNoError(t, err, "create push pipeline")
	if !created || view.Pipeline.Source != "push" || view.Pipeline.RefName != "main" || view.Pipeline.CommitSHA != branch[0].Hash {
		t.Fatalf("unexpected push pipeline: created=%v view=%+v", created, view.Pipeline)
	}
	if len(view.Jobs) != 2 || !strings.Contains(view.Jobs[0].ProjectJob.Payload, `"project_full_path":"core-team/gity"`) {
		t.Fatalf("unexpected push pipeline jobs: %+v", view.Jobs)
	}

	duplicate, created, err := env.Service.CreatePushPipeline(ctx, env.ProjectID, "main", branch[0].Hash)
	testutil.RequireNoError(t, err, "deduplicate push pipeline")
	if created || duplicate.Pipeline.ID != view.Pipeline.ID {
		t.Fatalf("expected duplicate trigger to reuse pipeline: created=%v duplicate=%+v", created, duplicate.Pipeline)
	}
}

func seedPipelineConfigCommit(ctx context.Context, t *testing.T, env pipelineTestEnv) {
	t.Helper()

	testutil.RequireNoError(t, env.GitRunner.InitBare(ctx, env.ProjectFullPath+".git", env.DefaultBranch), "init bare repo")
	testutil.RequireNoError(t, env.GitRunner.CreateFileCommit(ctx, env.ProjectFullPath+".git", gitexec.CreateFileCommitInput{
		BranchName:  "main",
		FilePath:    ".gity-ci.plano",
		Content:     pipelineConfig(),
		Message:     "Add CI config",
		AuthorName:  "Gity Test",
		AuthorEmail: "test@gity.dev",
	}), "create ci config commit")
}
