package pipeline_test

import (
	"context"
	"testing"
	"time"

	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
	projectjobrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_job"
	projectpipelinerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_pipeline"
)

func TestProjectPipelineFailureCancelsPendingJobs(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	env := setupPipelineTest(ctx, t)
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
	env := setupPipelineTest(ctx, t)
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
	env := setupPipelineTest(ctx, t)
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
