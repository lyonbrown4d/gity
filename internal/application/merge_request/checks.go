package mergerequest

import (
	"context"
	"errors"
	"log/slog"
	"strings"

	setx "github.com/arcgolabs/collectionx/set"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	"github.com/samber/oops"
)

func (s *Service) triggerTargetBranchPipeline(ctx context.Context, project projectdomain.Project, mr mergedomain.ProjectMergeRequest) {
	if s.pipelineSvc == nil {
		return
	}
	branch, err := s.resolveBranch(ctx, project, mr.TargetBranch)
	if err != nil || branch.Hash == "" {
		return
	}
	if _, _, err := s.pipelineSvc.CreatePushPipeline(ctx, project.ID, branch.Name, branch.Hash); err != nil {
		wrapped := oops.In("merge_request").With("project_id", project.ID, "merge_request_id", mr.ID, "target_branch", branch.Name, "commit_sha", branch.Hash).Wrapf(err, "trigger target branch pipeline")
		s.warn("trigger target branch pipeline failed", slog.String("error", wrapped.Error()))
	}
}

func (s *Service) evaluateChecks(ctx context.Context, project projectdomain.Project, mr mergedomain.ProjectMergeRequest) (CheckStatusView, error) {
	branch, err := s.resolveBranch(ctx, project, mr.SourceBranch)
	if err != nil {
		return CheckStatusView{}, err
	}
	view := CheckStatusView{
		MergeRequest:    mr,
		SourceBranch:    branch.Name,
		SourceCommitSHA: branch.Hash,
		TargetBranch:    mr.TargetBranch,
		Mergeable:       true,
		Status:          "not_required",
	}
	if protectionErr := s.applyTargetBranchProtection(ctx, project.ID, mr.TargetBranch, &view); protectionErr != nil {
		return CheckStatusView{}, protectionErr
	}
	ciRequired, err := s.hasCIConfig(ctx, project, branch.Hash)
	if err != nil {
		return CheckStatusView{}, err
	}
	if !view.pipelineIsRequired(ciRequired) {
		return s.evaluateRequiredApprovals(ctx, project, mr, view)
	}
	view, err = s.evaluateRequiredPipeline(ctx, project, mr, branch, view)
	if err != nil {
		return CheckStatusView{}, err
	}
	return s.evaluateRequiredApprovals(ctx, project, mr, view)
}

func (s *Service) applyTargetBranchProtection(ctx context.Context, projectID int64, targetBranch string, view *CheckStatusView) error {
	protection, protected, err := s.targetBranchProtection(ctx, projectID, targetBranch)
	if err != nil {
		return err
	}
	if protected {
		view.TargetBranchProtected = true
		view.RequireMergeRequest = protection.RequiresMergeRequest()
		view.RequirePipelineSuccess = protection.RequiresPipelineSuccess()
		view.MergeAccessLevel = projectdomain.NormalizeProjectBranchProtectionAccessLevel(protection.MergeAccessLevel, projectdomain.ProjectBranchProtectionAccessMaintainer)
		if view.RequireMergeRequest {
			view.RequireApproval = true
			view.RequiredApprovals = 1
		}
	}
	return nil
}

func (view CheckStatusView) pipelineIsRequired(ciRequired bool) bool {
	return ciRequired || view.RequirePipelineSuccess
}

func (s *Service) evaluateRequiredPipeline(ctx context.Context, project projectdomain.Project, mr mergedomain.ProjectMergeRequest, branch gitports.Branch, view CheckStatusView) (CheckStatusView, error) {
	view.Required = true
	view.PipelineRequired = true
	view.Mergeable = false
	if s.pipelineRepo == nil {
		view.Status = "missing"
		view = addMergeCheckBlocker(view, CheckBlockerView{
			Code:     mergeCheckBlockerPipelineRepoMissing,
			Category: mergeCheckBlockerCategoryPipeline,
			Message:  "pipeline repository is not configured",
		})
		return view, nil
	}
	pipeline, err := s.pipelineRepo.GetLatestByProjectRefCommit(ctx, project.ID, branch.Name, branch.Hash)
	if err != nil {
		if errors.Is(err, gitports.ErrNotFound) {
			view.Status = "missing"
			view = addMergeCheckBlocker(view, CheckBlockerView{
				Code:     mergeCheckBlockerPipelineMissing,
				Category: mergeCheckBlockerCategoryPipeline,
				Message:  "required pipeline is missing",
			})
			return view, nil
		}
		return CheckStatusView{}, oops.In("merge_request").With("project_id", project.ID, "merge_request_id", mr.ID, "ref", branch.Name, "commit_sha", branch.Hash).Wrapf(err, "load merge request pipeline")
	}
	view.Pipeline = &pipeline
	view.Status = pipeline.Status
	view.Mergeable = pipeline.Status == gitports.ProjectPipelineStatusSucceeded
	if !view.Mergeable {
		view = addMergeCheckBlocker(view, CheckBlockerView{
			Code:     mergeCheckBlockerPipelineNotSucceeded,
			Category: mergeCheckBlockerCategoryPipeline,
			Message:  "pipeline status is " + pipeline.Status,
		})
	}
	return view, nil
}

func (s *Service) evaluateRequiredApprovals(ctx context.Context, project projectdomain.Project, mr mergedomain.ProjectMergeRequest, view CheckStatusView) (CheckStatusView, error) {
	checks, approvalCount, err := s.evaluateApprovalRules(ctx, project, mr, view)
	if err != nil {
		return CheckStatusView{}, err
	}
	view.ApprovalRules = checks
	view.ApprovalCount = approvalCount
	view.RequiredApprovals = totalRequiredApprovals(checks)
	if len(checks) == 0 {
		return view, nil
	}
	view.RequireApproval = true
	view.Required = true
	blocked := false
	for _, check := range checks {
		if !check.Satisfied {
			blocked = true
			view = blockMergeCheck(view, CheckBlockerView{
				Code:     mergeCheckBlockerApprovalRuleUnsatisfied,
				Category: mergeCheckBlockerCategoryApproval,
				Message:  check.BlockingReason,
			})
		}
	}
	if blocked {
		return view, nil
	}
	return passMergeCheck(view), nil
}

func normalizedRequiredApprovals(value int) int {
	if value > 0 {
		return value
	}
	return 1
}

func (s *Service) nonAuthorApproverIDs(ctx context.Context, mr mergedomain.ProjectMergeRequest) (*setx.Set[int64], error) {
	approvers := setx.NewSet[int64]()
	if s.approvalRepo == nil {
		return approvers, nil
	}
	approvals, err := s.approvalRepo.ListByMergeRequestID(ctx, mr.ID)
	if err != nil {
		return nil, oops.In("merge_request").With("merge_request_id", mr.ID).Wrapf(err, "list merge request approvals for checks")
	}
	approvalValues := approvals.Values()
	for index := range approvalValues {
		approval := approvalValues[index]
		if approval.UserID > 0 && approval.UserID != mr.AuthorUserID {
			approvers.Add(approval.UserID)
		}
	}
	return approvers, nil
}

func (s *Service) evaluateApprovalRules(ctx context.Context, project projectdomain.Project, mr mergedomain.ProjectMergeRequest, view CheckStatusView) ([]ApprovalRuleCheck, int, error) {
	approvers, err := s.nonAuthorApproverIDs(ctx, mr)
	if err != nil {
		return nil, 0, err
	}
	checks := make([]ApprovalRuleCheck, 0)
	if view.RequireApproval {
		checks = append(checks, evaluateApprovalRuleCheck(ApprovalRuleCheck{
			Name:              "Protected branch approval",
			TargetBranch:      mr.TargetBranch,
			ApprovalsRequired: normalizedRequiredApprovals(view.RequiredApprovals),
		}, approvers))
	}
	rules, err := s.matchApprovalRules(ctx, project.ID, mr.TargetBranch)
	if err != nil {
		return nil, 0, err
	}
	cache := codeOwnerIDCache{}
	for index := range rules {
		check, include, ruleErr := s.approvalRuleCheck(ctx, project, mr, rules[index], approvers, &cache)
		if ruleErr != nil {
			return nil, 0, ruleErr
		}
		if include {
			checks = append(checks, check)
		}
	}
	return checks, approvers.Len(), nil
}

func (s *Service) matchApprovalRules(ctx context.Context, projectID int64, targetBranch string) ([]mergedomain.ProjectMergeRequestApprovalRule, error) {
	if s.approvalRuleRepo == nil {
		return nil, nil
	}
	rules, err := s.approvalRuleRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, oops.In("merge_request").With("project_id", projectID).Wrapf(err, "list approval rules for checks")
	}
	matched := make([]mergedomain.ProjectMergeRequestApprovalRule, 0, rules.Len())
	ruleValues := rules.Values()
	for index := range ruleValues {
		rule := ruleValues[index]
		if branchPatternMatches(rule.TargetBranch, targetBranch) {
			matched = append(matched, rule)
		}
	}
	return matched, nil
}

func evaluateApprovalRuleCheck(check ApprovalRuleCheck, approvers *setx.Set[int64]) ApprovalRuleCheck {
	check.ApprovalsRequired = normalizedRequiredApprovals(check.ApprovalsRequired)
	if len(check.EligibleUserIDs) == 0 {
		check.ApprovalCount = approvers.Len()
	} else {
		eligibleApprovers := setx.NewSet[int64](check.EligibleUserIDs...)
		check.ApprovalCount = approvers.Intersect(eligibleApprovers).Len()
	}
	check.Satisfied = check.ApprovalCount >= check.ApprovalsRequired
	if !check.Satisfied {
		check.BlockingReason = "approval rule " + check.Name + " requires approval"
	}
	return check
}

func totalRequiredApprovals(checks []ApprovalRuleCheck) int {
	total := 0
	for _, check := range checks {
		total += normalizedRequiredApprovals(check.ApprovalsRequired)
	}
	return total
}

func (s *Service) resolveCodeOwnerIDs(ctx context.Context, project projectdomain.Project, mr mergedomain.ProjectMergeRequest) ([]int64, error) {
	owners, err := s.loadCodeOwners(ctx, project, mr.TargetBranch)
	if err != nil || len(owners) == 0 {
		return nil, err
	}
	changedFiles, err := s.changedFiles(ctx, project, mr)
	if err != nil {
		return nil, err
	}
	usernames := matchedCodeOwnerUsernames(owners, changedFiles)
	ids := setx.NewSetWithCapacity[int64](len(usernames))
	for _, username := range usernames {
		user, userErr := s.userRepo.GetByUsername(ctx, username)
		if userErr != nil {
			continue
		}
		ids.Add(user.ID)
	}
	return ids.Values(), nil
}

func (s *Service) changedFiles(ctx context.Context, project projectdomain.Project, mr mergedomain.ProjectMergeRequest) ([]string, error) {
	diff, err := s.gitRunner.DiffBranches(ctx, project.FullPath+".git", mr.TargetBranch, mr.SourceBranch)
	if err != nil {
		return nil, mapGitExecError(err)
	}
	files := setx.NewSet[string]()
	for line := range strings.SplitSeq(diff, "\n") {
		if !strings.HasPrefix(line, "+++ b/") {
			continue
		}
		filePath := strings.TrimSpace(strings.TrimPrefix(line, "+++ b/"))
		if filePath != "" && filePath != "/dev/null" {
			files.Add(filePath)
		}
	}
	return files.Values(), nil
}
