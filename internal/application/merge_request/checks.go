package mergerequest

import (
	"context"
	"encoding/base64"
	"errors"
	"fmt"
	"log/slog"
	"path"
	"regexp"
	"strings"

	setx "github.com/arcgolabs/collectionx/set"
	"github.com/bmatcuk/doublestar/v4"
	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
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
		view.BlockingReason = "pipeline repository is not configured"
		return view, nil
	}
	pipeline, err := s.pipelineRepo.GetLatestByProjectRefCommit(ctx, project.ID, branch.Name, branch.Hash)
	if err != nil {
		if errors.Is(err, gitports.ErrNotFound) {
			view.Status = "missing"
			view.BlockingReason = "required pipeline is missing"
			return view, nil
		}
		return CheckStatusView{}, oops.In("merge_request").With("project_id", project.ID, "merge_request_id", mr.ID, "ref", branch.Name, "commit_sha", branch.Hash).Wrapf(err, "load merge request pipeline")
	}
	view.Pipeline = &pipeline
	view.Status = pipeline.Status
	view.Mergeable = pipeline.Status == gitports.ProjectPipelineStatusSucceeded
	if !view.Mergeable {
		view.BlockingReason = "pipeline status is " + pipeline.Status
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
	for _, check := range checks {
		if !check.Satisfied {
			return blockMergeCheck(view, check.BlockingReason), nil
		}
	}
	return passMergeCheck(view), nil
}

func normalizedRequiredApprovals(value int) int {
	if value > 0 {
		return value
	}
	return 1
}

func (s *Service) nonAuthorApprovalCount(ctx context.Context, mr mergedomain.ProjectMergeRequest) (int, error) {
	approvers, err := s.nonAuthorApproverIDs(ctx, mr)
	if err != nil {
		return 0, err
	}
	return approvers.Len(), nil
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
	for _, approval := range approvals.Values() {
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
	var codeOwnerIDs []int64
	for _, rule := range rules {
		eligibleUserIDs, decodeErr := decodeApprovalRuleUserIDs(rule.EligibleUserIDs)
		if decodeErr != nil {
			return nil, 0, oops.In("merge_request").With("project_id", project.ID, "rule_id", rule.ID).Wrapf(decodeErr, "decode approval rule eligible users")
		}
		if rule.RequiresCodeOwner() {
			if codeOwnerIDs == nil {
				codeOwnerIDs, err = s.resolveCodeOwnerIDs(ctx, project, mr)
				if err != nil {
					return nil, 0, err
				}
			}
			if len(codeOwnerIDs) == 0 {
				continue
			}
			eligibleUserIDs = codeOwnerIDs
		}
		checks = append(checks, evaluateApprovalRuleCheck(ApprovalRuleCheck{
			RuleID:            rule.ID,
			Name:              rule.Name,
			TargetBranch:      rule.TargetBranch,
			ApprovalsRequired: normalizedRequiredApprovals(rule.ApprovalsRequired),
			EligibleUserIDs:   eligibleUserIDs,
			CodeOwner:         rule.RequiresCodeOwner(),
		}, approvers))
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
	for _, rule := range rules.Values() {
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

func branchPatternMatches(pattern, branch string) bool {
	pattern = strings.TrimSpace(pattern)
	branch = strings.TrimSpace(branch)
	if pattern == "" || pattern == "*" {
		return true
	}
	matched, err := doublestar.Match(pattern, branch)
	if err == nil {
		return matched
	}
	return pattern == branch
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

type codeOwnerRule struct {
	Pattern   string
	Usernames []string
}

func (s *Service) loadCodeOwners(ctx context.Context, project projectdomain.Project, targetBranch string) ([]codeOwnerRule, error) {
	for _, codeOwnersPath := range []string{"CODEOWNERS", ".gitlab/CODEOWNERS", "docs/CODEOWNERS"} {
		blob, err := s.gitRepo.GetBlob(ctx, project.FullPath+".git", targetBranch, project.DefaultBranch, codeOwnersPath)
		if err == nil {
			content, contentErr := codeOwnersBlobContent(blob)
			if contentErr != nil {
				return nil, oops.In("merge_request").With("project_id", project.ID, "path", codeOwnersPath).Wrapf(contentErr, "read CODEOWNERS")
			}
			return parseCodeOwners(content), nil
		}
		if !errors.Is(err, gitports.ErrPathNotFound) && !errors.Is(err, gitports.ErrReferenceNotFound) && !errors.Is(err, gitports.ErrEmptyRepository) {
			return nil, oops.In("merge_request").With("project_id", project.ID, "path", codeOwnersPath).Wrapf(err, "load CODEOWNERS")
		}
	}
	return nil, nil
}

func codeOwnersBlobContent(blob gitports.Blob) (string, error) {
	if blob.Encoding == "" || blob.Encoding == "utf-8" {
		return blob.Content, nil
	}
	if blob.Encoding == "base64" {
		data, err := base64.StdEncoding.DecodeString(blob.Content)
		if err != nil {
			return "", err
		}
		return string(data), nil
	}
	return "", fmt.Errorf("unsupported CODEOWNERS encoding %s", blob.Encoding)
}

var codeOwnerUsernamePattern = regexp.MustCompile(`@([A-Za-z0-9][A-Za-z0-9_.-]*)`)

func parseCodeOwners(content string) []codeOwnerRule {
	rules := make([]codeOwnerRule, 0)
	for _, line := range strings.Split(content, "\n") {
		line = strings.TrimSpace(stripCodeOwnerComment(line))
		if line == "" || strings.HasPrefix(line, "[") || strings.HasPrefix(line, "!") {
			continue
		}
		fields := strings.Fields(line)
		if len(fields) < 2 {
			continue
		}
		usernames := make([]string, 0, len(fields)-1)
		for _, owner := range fields[1:] {
			matches := codeOwnerUsernamePattern.FindStringSubmatch(owner)
			if len(matches) == 2 {
				usernames = append(usernames, strings.ToLower(matches[1]))
			}
		}
		if len(usernames) > 0 {
			rules = append(rules, codeOwnerRule{Pattern: fields[0], Usernames: usernames})
		}
	}
	return rules
}

func stripCodeOwnerComment(line string) string {
	if index := strings.Index(line, "#"); index >= 0 {
		return line[:index]
	}
	return line
}

func (s *Service) changedFiles(ctx context.Context, project projectdomain.Project, mr mergedomain.ProjectMergeRequest) ([]string, error) {
	diff, err := s.gitRunner.DiffBranches(ctx, project.FullPath+".git", mr.TargetBranch, mr.SourceBranch)
	if err != nil {
		return nil, mapGitExecError(err)
	}
	files := setx.NewSet[string]()
	for _, line := range strings.Split(diff, "\n") {
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

func matchedCodeOwnerUsernames(rules []codeOwnerRule, files []string) []string {
	usernames := setx.NewSet[string]()
	for _, file := range files {
		var matched *codeOwnerRule
		for _, rule := range rules {
			if codeOwnerPatternMatches(rule.Pattern, file) {
				current := rule
				matched = &current
			}
		}
		if matched == nil {
			continue
		}
		for _, username := range matched.Usernames {
			usernames.Add(username)
		}
	}
	return usernames.Values()
}

func codeOwnerPatternMatches(pattern, filePath string) bool {
	pattern = strings.TrimSpace(strings.TrimPrefix(pattern, "/"))
	filePath = strings.TrimSpace(strings.ReplaceAll(filePath, "\\", "/"))
	if pattern == "" {
		return false
	}
	if pattern == "*" {
		return true
	}
	candidates := []string{pattern}
	if !strings.Contains(pattern, "/") {
		candidates = append(candidates, path.Join("**", pattern), path.Join("**", pattern+"*"))
	}
	if strings.HasSuffix(pattern, "/") {
		candidates = append(candidates, path.Join(pattern, "**"))
	}
	for _, candidate := range candidates {
		matched, err := doublestar.PathMatch(candidate, filePath)
		if err == nil && matched {
			return true
		}
	}
	return pattern == filePath
}

func blockMergeCheck(view CheckStatusView, reason string) CheckStatusView {
	view.Mergeable = false
	if view.BlockingReason == "" {
		view.BlockingReason = reason
		view.Status = "blocked"
	}
	return view
}

func passMergeCheck(view CheckStatusView) CheckStatusView {
	if view.BlockingReason != "" {
		return view
	}
	view.Mergeable = true
	if view.Status == "not_required" {
		view.Status = "passed"
	}
	return view
}

func (s *Service) targetBranchProtection(ctx context.Context, projectID int64, branchName string) (projectdomain.ProjectBranchProtection, bool, error) {
	if s.branchRepo == nil {
		return projectdomain.ProjectBranchProtection{}, false, nil
	}
	protection, err := s.branchRepo.MatchByProjectAndBranch(ctx, projectID, branchName)
	if err == nil {
		return protection, true, nil
	}
	if errors.Is(err, gitports.ErrNotFound) {
		return projectdomain.ProjectBranchProtection{}, false, nil
	}
	return projectdomain.ProjectBranchProtection{}, false, oops.In("merge_request").With("project_id", projectID, "target_branch", branchName).Wrapf(err, "check target branch protection")
}

func (s *Service) hasCIConfig(ctx context.Context, project projectdomain.Project, commitSHA string) (bool, error) {
	_, err := s.gitRepo.GetBlob(ctx, project.FullPath+".git", commitSHA, project.DefaultBranch, defaultCIConfigPath)
	if err == nil {
		return true, nil
	}
	if errors.Is(err, gitports.ErrPathNotFound) || errors.Is(err, gitports.ErrReferenceNotFound) || errors.Is(err, gitports.ErrEmptyRepository) {
		return false, nil
	}
	return false, oops.In("merge_request").With("project_id", project.ID, "commit_sha", commitSHA, "path", defaultCIConfigPath).Wrapf(err, "check ci config")
}

func (s *Service) resolveBranch(ctx context.Context, project projectdomain.Project, branch string) (gitports.Branch, error) {
	branches, err := s.gitRepo.ListBranches(ctx, project.FullPath+".git", project.DefaultBranch)
	if err != nil {
		return gitports.Branch{}, oops.In("merge_request").With("project_id", project.ID, "branch", branch).Wrapf(err, "list branches")
	}
	for _, item := range branches {
		if item.Name == branch {
			return item, nil
		}
	}
	return gitports.Branch{}, apperror.NotFound("merge request branch not found", fmt.Errorf("branch %s not found", branch))
}

func (s *Service) ensureBranchExists(ctx context.Context, project projectdomain.Project, branch string) error {
	_, err := s.resolveBranch(ctx, project, branch)
	return err
}

func mapGitExecError(err error) error {
	switch {
	case errors.Is(err, gitports.ErrMergeConflict):
		return apperror.Conflict("merge conflict", err)
	case errors.Is(err, gitports.ErrSourceReferenceNotFound):
		return apperror.NotFound("git reference not found", err)
	case errors.Is(err, gitports.ErrInvalidBranchName):
		return apperror.BadRequest("invalid branch name", err)
	default:
		return err
	}
}
