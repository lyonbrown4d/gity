package mergerequest_test

import (
	"testing"

	mergerequestservice "github.com/DaiYuANg/gity/internal/application/merge_request"
	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_exec"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_repo"
	projectpipelinerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_pipeline"
	"github.com/DaiYuANg/gity/internal/testutil"
)

func TestMergeRequestMergeRequiresSuccessfulPipelineWhenCIConfigExists(t *testing.T) {
	t.Parallel()

	fixture := newMergeRequestFixture(t, true)
	sourceBranch := addMergeRequestCIConfig(t, fixture)
	mrIID := assertCreateMergeRequest(t, fixture, "merge feature with checks")
	assertMissingMergeRequestChecks(t, fixture, mrIID)
	assertMergeRequestBlocked(t, fixture, mrIID)
	createFailedPipelineAndMarkSucceeded(t, fixture, sourceBranch.Hash, mrIID)
	assertMergeAfterSuccessfulPipeline(t, fixture, mrIID)
	assertTargetBranchPipeline(t, fixture)
}

func TestMergeRequestMergeRequiresProtectedTargetBranchPipeline(t *testing.T) {
	t.Parallel()

	fixture := newMergeRequestFixture(t, true)
	requireTargetBranchPipeline(t, fixture, "main")
	mrIID := assertCreateMergeRequest(t, fixture, "merge protected target")
	sourceBranch := findBranch(fixture.ctx, t, fixture.gitRepository, fixture.projectFullPath+".git", fixture.projectDefaultBranch, "feature")
	assertProtectedTargetBranchChecks(t, fixture, mrIID)
	assertMergeRequestBlocked(t, fixture, mrIID)
	createFailedPipelineAndMarkSucceeded(t, fixture, sourceBranch.Hash, mrIID)
	assertMergeAfterSuccessfulPipeline(t, fixture, mrIID)
}

func addMergeRequestCIConfig(t *testing.T, fixture mergeRequestFixture) gitrepo.Branch {
	t.Helper()

	testutil.RequireNoError(t, fixture.runner.CreateFileCommit(fixture.ctx, fixture.projectFullPath+".git", gitexec.CreateFileCommitInput{
		BranchName:  "feature",
		FilePath:    ".gity-ci.plano",
		Content:     mergeRequestCIConfig(),
		Message:     "Add CI config",
		AuthorName:  "Gity Test",
		AuthorEmail: "test@gity.dev",
	}), "add ci config")
	return findBranch(fixture.ctx, t, fixture.gitRepository, fixture.projectFullPath+".git", fixture.projectDefaultBranch, "feature")
}

func assertMissingMergeRequestChecks(t *testing.T, fixture mergeRequestFixture, mrIID int64) {
	t.Helper()

	checks := testutil.Must(fixture.mergeRequestService.GetChecks(fixture.ctx, fixture.projectID, mrIID))
	if !checks.PipelineRequired || !checks.Required || checks.Mergeable || checks.Status != "missing" {
		t.Fatalf("expected missing required checks: %+v", checks)
	}
}

func requireTargetBranchPipeline(t *testing.T, fixture mergeRequestFixture, branchName string) {
	t.Helper()

	protection := testutil.Must(fixture.projectService.UpsertBranchProtection(fixture.ctx, fixture.projectID, projectservice.BranchProtectionInput{
		BranchName:             branchName,
		RuleType:               "exact",
		PushAccessLevel:        "no_one",
		MergeAccessLevel:       "maintainer",
		RequireMergeRequest:    true,
		RequirePipelineSuccess: true,
	}))
	if !protection.RequirePipelineSuccess || !protection.RequireMergeRequest {
		t.Fatalf("unexpected protected target branch rule: %+v", protection)
	}
}

func assertProtectedTargetBranchChecks(t *testing.T, fixture mergeRequestFixture, mrIID int64) {
	t.Helper()

	checks := testutil.Must(fixture.mergeRequestService.GetChecks(fixture.ctx, fixture.projectID, mrIID))
	if !checks.TargetBranchProtected || !checks.RequireMergeRequest || !checks.RequirePipelineSuccess {
		t.Fatalf("expected protected target branch checks: %+v", checks)
	}
	if !checks.PipelineRequired || !checks.Required || checks.Mergeable || checks.Status != "missing" {
		t.Fatalf("expected protected target branch missing pipeline checks: %+v", checks)
	}
}

func assertMergeRequestBlocked(t *testing.T, fixture mergeRequestFixture, mrIID int64) {
	t.Helper()

	if _, mergeErr := fixture.mergeRequestService.Merge(fixture.ctx, fixture.projectID, mrIID, mergerequestservice.MergeInput{AuthorName: "Gity Test", AuthorEmail: "test@gity.dev"}); mergeErr == nil {
		t.Fatalf("expected merge to be blocked before pipeline exists")
	}
}

func createFailedPipelineAndMarkSucceeded(t *testing.T, fixture mergeRequestFixture, sourceHash string, mrIID int64) {
	t.Helper()

	pipeline := testutil.Must(fixture.pipelineRepository.Create(fixture.ctx, projectpipelinerepo.CreateInput{
		ProjectID:     fixture.projectID,
		Name:          "merge-request",
		Source:        "push",
		RefName:       "feature",
		CommitSHA:     sourceHash,
		Status:        projectpipelinerepo.StatusFailed,
		ConfigSource:  ".gity-ci.plano",
		ConfigContent: mergeRequestCIConfig(),
	}))
	checks := testutil.Must(fixture.mergeRequestService.GetChecks(fixture.ctx, fixture.projectID, mrIID))
	if checks.Mergeable || checks.Status != projectpipelinerepo.StatusFailed || checks.Pipeline == nil {
		t.Fatalf("expected failed checks: %+v", checks)
	}
	testutil.RequireNoError(t, fixture.pipelineRepository.UpdateStatus(fixture.ctx, pipeline, projectpipelinerepo.StatusSucceeded), "mark pipeline succeeded")
}

func assertMergeAfterSuccessfulPipeline(t *testing.T, fixture mergeRequestFixture, mrIID int64) {
	t.Helper()
	assertMergeRequestMerge(t, fixture, mrIID)
}

func assertTargetBranchPipeline(t *testing.T, fixture mergeRequestFixture) {
	t.Helper()

	targetBranch := findBranch(fixture.ctx, t, fixture.gitRepository, fixture.projectFullPath+".git", fixture.projectDefaultBranch, "main")
	targetPipeline := testutil.Must(fixture.pipelineRepository.GetLatestByProjectRefCommit(fixture.ctx, fixture.projectID, "main", targetBranch.Hash))
	if targetPipeline.Source != "push" || targetPipeline.CommitSHA != targetBranch.Hash {
		t.Fatalf("unexpected target branch pipeline: %+v", targetPipeline)
	}
}
