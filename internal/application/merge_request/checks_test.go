package mergerequest_test

import (
	"testing"

	mergerequestservice "github.com/lyonbrown4d/gity/internal/application/merge_request"
	projectservice "github.com/lyonbrown4d/gity/internal/application/project"
	"github.com/lyonbrown4d/gity/internal/infrastructure/git_exec"
	"github.com/lyonbrown4d/gity/internal/infrastructure/git_repo"
	projectpipelinerepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_pipeline"
	"github.com/lyonbrown4d/gity/internal/testutil"
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
	assertMergeRequestBlocked(t, fixture, mrIID)
	assertApproveMergeRequest(t, fixture, mrIID)
	assertMergeAfterSuccessfulPipeline(t, fixture, mrIID)
}

func TestMergeRequestApprovalRuleRequiresEligibleApprover(t *testing.T) {
	t.Parallel()

	fixture := newMergeRequestFixture(t, true)
	mrIID := assertCreateMergeRequest(t, fixture, "merge with approval rule")
	rule := testutil.Must(fixture.mergeRequestService.CreateApprovalRule(fixture.ctx, fixture.projectID, mergerequestservice.ApprovalRuleInput{
		Name:              "Maintainer review",
		TargetBranch:      "main",
		ApprovalsRequired: 1,
		EligibleUserIDs:   []int64{fixture.reviewerID},
	}))
	if rule.ID == 0 || len(rule.EligibleUserIDs) != 1 {
		t.Fatalf("unexpected approval rule: %+v", rule)
	}
	checks := testutil.Must(fixture.mergeRequestService.GetChecks(fixture.ctx, fixture.projectID, mrIID))
	if checks.Mergeable || checks.RequiredApprovals != 1 || len(checks.ApprovalRules) != 1 {
		t.Fatalf("expected approval rule to block merge: %+v", checks)
	}
	assertApproveMergeRequest(t, fixture, mrIID)
	checks = testutil.Must(fixture.mergeRequestService.GetChecks(fixture.ctx, fixture.projectID, mrIID))
	if !checks.Mergeable || !checks.ApprovalRules[0].Satisfied {
		t.Fatalf("expected approval rule to pass: %+v", checks)
	}
}

func TestMergeRequestCodeOwnersApprovalRule(t *testing.T) {
	t.Parallel()

	fixture := newMergeRequestFixture(t, true)
	addCodeOwners(t, fixture, "* @bob\n")
	mrIID := assertCreateMergeRequest(t, fixture, "merge with code owners")
	rule := testutil.Must(fixture.mergeRequestService.CreateApprovalRule(fixture.ctx, fixture.projectID, mergerequestservice.ApprovalRuleInput{
		Name:              "Code owners",
		TargetBranch:      "main",
		ApprovalsRequired: 1,
		CodeOwner:         true,
	}))
	if !rule.CodeOwner {
		t.Fatalf("expected code owner rule: %+v", rule)
	}
	checks := testutil.Must(fixture.mergeRequestService.GetChecks(fixture.ctx, fixture.projectID, mrIID))
	if checks.Mergeable || len(checks.ApprovalRules) != 1 || len(checks.ApprovalRules[0].EligibleUserIDs) != 1 {
		t.Fatalf("expected code owner approval rule to block merge: %+v", checks)
	}
	assertApproveMergeRequest(t, fixture, mrIID)
	checks = testutil.Must(fixture.mergeRequestService.GetChecks(fixture.ctx, fixture.projectID, mrIID))
	if !checks.Mergeable || !checks.ApprovalRules[0].Satisfied {
		t.Fatalf("expected code owner approval rule to pass: %+v", checks)
	}
}

func TestMergeRequestCodeOwnersUsesLastMatchingRule(t *testing.T) {
	t.Parallel()

	fixture := newMergeRequestFixture(t, true)
	addCodeOwners(t, fixture, "[Core]\n* @alice\n*.txt @bob\n")
	mrIID := assertCreateMergeRequest(t, fixture, "merge with last matching code owners")
	testutil.Must(fixture.mergeRequestService.CreateApprovalRule(fixture.ctx, fixture.projectID, mergerequestservice.ApprovalRuleInput{
		Name:              "Code owners",
		TargetBranch:      "main",
		ApprovalsRequired: 1,
		CodeOwner:         true,
	}))
	checks := testutil.Must(fixture.mergeRequestService.GetChecks(fixture.ctx, fixture.projectID, mrIID))
	if len(checks.ApprovalRules) != 1 {
		t.Fatalf("expected one approval rule: %+v", checks)
	}
	owners := checks.ApprovalRules[0].EligibleUserIDs
	if len(owners) != 1 || owners[0] != fixture.reviewerID {
		t.Fatalf("expected last matching code owner to win: %+v", checks.ApprovalRules[0])
	}
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

func addCodeOwners(t *testing.T, fixture mergeRequestFixture, content string) {
	t.Helper()

	testutil.RequireNoError(t, fixture.runner.CreateFileCommit(fixture.ctx, fixture.projectFullPath+".git", gitexec.CreateFileCommitInput{
		BranchName:  "main",
		FilePath:    "CODEOWNERS",
		Content:     content,
		Message:     "Add CODEOWNERS",
		AuthorName:  "Gity Test",
		AuthorEmail: "test@gity.dev",
	}), "add CODEOWNERS")
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
	if !checks.TargetBranchProtected || !checks.RequireMergeRequest || !checks.RequirePipelineSuccess || !checks.RequireApproval || checks.RequiredApprovals != 1 {
		t.Fatalf("expected protected target branch checks: %+v", checks)
	}
	if !checks.PipelineRequired || !checks.Required || checks.Mergeable || checks.Status != "missing" {
		t.Fatalf("expected protected target branch missing pipeline checks: %+v", checks)
	}
}

func assertApproveMergeRequest(t *testing.T, fixture mergeRequestFixture, mrIID int64) {
	t.Helper()

	approvals := testutil.Must(fixture.mergeRequestService.Approve(fixture.ctx, fixture.projectID, mrIID, mergerequestservice.ApprovalInput{UserID: fixture.reviewerID}))
	if len(approvals.Approvals) != 1 {
		t.Fatalf("expected merge request approval: %+v", approvals)
	}
	checks := testutil.Must(fixture.mergeRequestService.GetChecks(fixture.ctx, fixture.projectID, mrIID))
	if checks.ApprovalCount != 1 {
		t.Fatalf("expected approved checks: %+v", checks)
	}
}

func assertMergeRequestBlocked(t *testing.T, fixture mergeRequestFixture, mrIID int64) {
	t.Helper()

	if _, mergeErr := fixture.mergeRequestService.Merge(fixture.ctx, fixture.projectID, mrIID, mergerequestservice.MergeInput{AuthorName: "Gity Test", AuthorEmail: "test@gity.dev", ActorUserID: fixture.ownerID}); mergeErr == nil {
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
