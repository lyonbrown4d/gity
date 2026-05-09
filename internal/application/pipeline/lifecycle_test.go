package pipeline_test

import (
	"context"
	"testing"
	"time"

	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
	projectjobrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_job"
	projectpipelinerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_pipeline"
	"github.com/DaiYuANg/gity/internal/testutil"
)

func TestProjectPipelineFailureCancelsPendingJobs(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	env := setupPipelineTest(ctx, t)
	created := createTestPipeline(ctx, t, env, parallelPipelineConfig())
	claimedID := claimPipelineJobID(ctx, t, env, "runner-a")
	failAndRefreshPipelineJob(ctx, t, env, claimedID)
	view := testutil.Must(env.Service.GetPipeline(ctx, env.ProjectID, created.Pipeline.ID))
	if view.Pipeline.Status != projectpipelinerepo.StatusFailed {
		t.Fatalf("pipeline status = %s", view.Pipeline.Status)
	}
	assertFailedAndCanceledJobs(t, view)
}

func failAndRefreshPipelineJob(ctx context.Context, t *testing.T, env pipelineTestEnv, jobID int64) {
	t.Helper()

	_, failErr := env.JobService.FailProjectJob(ctx, env.ProjectID, jobID, "failed", time.Second)
	testutil.RequireNoError(t, failErr, "fail pipeline job")
	testutil.RequireNoError(t, env.Service.RefreshProjectJob(ctx, env.ProjectID, jobID), "refresh pipeline job")
}

func assertFailedAndCanceledJobs(t *testing.T, view pipelineservice.PipelineView) {
	t.Helper()

	canceled := 0
	failed := 0
	for index := range view.Jobs {
		item := &view.Jobs[index]
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
	env := setupPipelineTest(ctx, t)
	created := createTestPipeline(ctx, t, env, pipelineConfig())
	view := testutil.Must(env.Service.CancelPipeline(ctx, env.ProjectID, created.Pipeline.ID))
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
	env := setupPipelineTest(ctx, t)
	created := createTestPipeline(ctx, t, env, pipelineConfig())
	claimedID := claimPipelineJobID(ctx, t, env, "runner-a")
	failAndRefreshPipelineJob(ctx, t, env, claimedID)
	retried := testutil.Must(env.Service.RetryPipeline(ctx, env.ProjectID, created.Pipeline.ID))
	if retried.Pipeline.Status != projectpipelinerepo.StatusPending {
		t.Fatalf("pipeline status = %s", retried.Pipeline.Status)
	}
	assertRetriedJobsReset(ctx, t, env)
	assertRetriedJobClaimable(ctx, t, env, claimedID)
}

func assertRetriedJobsReset(ctx context.Context, t *testing.T, env pipelineTestEnv) {
	t.Helper()

	jobs := testutil.Must(env.JobService.ListProjectJobs(ctx, env.ProjectID))
	for index := range jobs {
		item := &jobs[index]
		if item.Status != projectjobrepo.StatusPending {
			t.Fatalf("expected pending job status after retry: %+v", item)
		}
		if item.Attempts != 0 {
			t.Fatalf("attempts should reset on retry: %+v", item)
		}
	}
}

func assertRetriedJobClaimable(ctx context.Context, t *testing.T, env pipelineTestEnv, expectedJobID int64) {
	t.Helper()

	reclaimed, ok, err := env.JobService.ClaimProjectJob(ctx, env.ProjectID, "runner-a", time.Minute)
	testutil.RequireNoError(t, err, "claim retried job")
	if !ok || reclaimed.ID != expectedJobID {
		t.Fatalf("expected retried first job to be claimable: %+v", reclaimed)
	}
}

func claimPipelineJobID(ctx context.Context, t *testing.T, env pipelineTestEnv, worker string) int64 {
	t.Helper()

	claimed, ok, err := env.JobService.ClaimProjectJob(ctx, env.ProjectID, worker, time.Minute)
	testutil.RequireNoError(t, err, "claim pipeline job")
	if !ok {
		t.Fatalf("expected pipeline job to be claimed")
	}
	return claimed.ID
}
