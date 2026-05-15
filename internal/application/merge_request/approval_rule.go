package mergerequest

import (
	"context"
	"errors"
	"strconv"
	"strings"

	setx "github.com/arcgolabs/collectionx/set"
	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
	"github.com/samber/oops"
)

type ApprovalRuleInput struct {
	Name              string  `json:"name"`
	TargetBranch      string  `json:"target_branch"`
	ApprovalsRequired int     `json:"approvals_required"`
	EligibleUserIDs   []int64 `json:"eligible_user_ids"`
	CodeOwner         bool    `json:"code_owner"`
}

type UpdateApprovalRuleInput struct {
	Name              *string  `json:"name"`
	TargetBranch      *string  `json:"target_branch"`
	ApprovalsRequired *int     `json:"approvals_required"`
	EligibleUserIDs   *[]int64 `json:"eligible_user_ids"`
	CodeOwner         *bool    `json:"code_owner"`
}

type ApprovalRuleView struct {
	ID                int64   `json:"id"`
	ProjectID         int64   `json:"project_id"`
	Name              string  `json:"name"`
	TargetBranch      string  `json:"target_branch"`
	ApprovalsRequired int     `json:"approvals_required"`
	EligibleUserIDs   []int64 `json:"eligible_user_ids"`
	CodeOwner         bool    `json:"code_owner"`
}

type ApprovalRulesView struct {
	ProjectID int64              `json:"project_id"`
	Rules     []ApprovalRuleView `json:"rules"`
}

type ApprovalRuleCheck struct {
	RuleID            int64   `json:"rule_id"`
	Name              string  `json:"name"`
	TargetBranch      string  `json:"target_branch"`
	ApprovalsRequired int     `json:"approvals_required"`
	ApprovalCount     int     `json:"approval_count"`
	EligibleUserIDs   []int64 `json:"eligible_user_ids"`
	CodeOwner         bool    `json:"code_owner"`
	Satisfied         bool    `json:"satisfied"`
	BlockingReason    string  `json:"blocking_reason,omitempty"`
}

func (s *Service) ListApprovalRules(ctx context.Context, projectID int64) (ApprovalRulesView, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return ApprovalRulesView{}, apperror.NotFound("project not found", err)
	}
	if s.approvalRuleRepo == nil {
		return ApprovalRulesView{}, oops.In("merge_request").With("project_id", projectID).New("merge request approval rule repository is not configured")
	}
	items, err := s.approvalRuleRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return ApprovalRulesView{}, oops.In("merge_request").With("project_id", projectID).Wrapf(err, "list merge request approval rules")
	}
	views := make([]ApprovalRuleView, 0, items.Len())
	itemValues := items.Values()
	for index := range itemValues {
		item := itemValues[index]
		view, viewErr := approvalRuleView(item)
		if viewErr != nil {
			return ApprovalRulesView{}, viewErr
		}
		views = append(views, view)
	}
	return ApprovalRulesView{ProjectID: projectID, Rules: views}, nil
}

func (s *Service) CreateApprovalRule(ctx context.Context, projectID int64, input ApprovalRuleInput) (ApprovalRuleView, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return ApprovalRuleView{}, apperror.NotFound("project not found", err)
	}
	if s.approvalRuleRepo == nil {
		return ApprovalRuleView{}, oops.In("merge_request").With("project_id", projectID).New("merge request approval rule repository is not configured")
	}
	normalized, err := s.normalizeApprovalRuleInput(ctx, projectID, input)
	if err != nil {
		return ApprovalRuleView{}, err
	}
	item, err := s.approvalRuleRepo.Create(ctx, normalized)
	if err != nil {
		return ApprovalRuleView{}, oops.In("merge_request").With("project_id", projectID, "name", input.Name).Wrapf(err, "create merge request approval rule")
	}
	return approvalRuleView(item)
}

func (s *Service) UpdateApprovalRule(ctx context.Context, projectID, ruleID int64, input UpdateApprovalRuleInput) (ApprovalRuleView, error) {
	if s.approvalRuleRepo == nil {
		return ApprovalRuleView{}, oops.In("merge_request").With("project_id", projectID, "rule_id", ruleID).New("merge request approval rule repository is not configured")
	}
	existing, err := s.approvalRuleRepo.GetByProjectAndID(ctx, projectID, ruleID)
	if err != nil {
		if errors.Is(err, gitports.ErrNotFound) {
			return ApprovalRuleView{}, apperror.NotFound("merge request approval rule not found", err)
		}
		return ApprovalRuleView{}, oops.In("merge_request").With("project_id", projectID, "rule_id", ruleID).Wrapf(err, "load merge request approval rule")
	}
	patch, err := s.normalizeUpdateApprovalRuleInput(ctx, projectID, existing, input)
	if err != nil {
		return ApprovalRuleView{}, err
	}
	updateErr := s.approvalRuleRepo.UpdateByID(ctx, ruleID, patch)
	if updateErr != nil {
		return ApprovalRuleView{}, oops.In("merge_request").With("project_id", projectID, "rule_id", ruleID).Wrapf(updateErr, "update merge request approval rule")
	}
	updated, err := s.approvalRuleRepo.GetByProjectAndID(ctx, projectID, ruleID)
	if err != nil {
		return ApprovalRuleView{}, oops.In("merge_request").With("project_id", projectID, "rule_id", ruleID).Wrapf(err, "reload merge request approval rule")
	}
	return approvalRuleView(updated)
}

func (s *Service) DeleteApprovalRule(ctx context.Context, projectID, ruleID int64) error {
	if s.approvalRuleRepo == nil {
		return oops.In("merge_request").With("project_id", projectID, "rule_id", ruleID).New("merge request approval rule repository is not configured")
	}
	if err := s.approvalRuleRepo.DeleteByProjectAndID(ctx, projectID, ruleID); err != nil {
		return oops.In("merge_request").With("project_id", projectID, "rule_id", ruleID).Wrapf(err, "delete merge request approval rule")
	}
	return nil
}

func (s *Service) normalizeApprovalRuleInput(ctx context.Context, projectID int64, input ApprovalRuleInput) (gitports.CreateProjectMergeRequestApprovalRuleInput, error) {
	name := strings.TrimSpace(input.Name)
	if name == "" {
		return gitports.CreateProjectMergeRequestApprovalRuleInput{}, apperror.BadRequest("approval rule name is required", oops.In("merge_request").With("project_id", projectID).New("approval rule name is required"))
	}
	targetBranch := strings.TrimSpace(input.TargetBranch)
	if targetBranch == "" {
		targetBranch = "*"
	}
	required := normalizedRequiredApprovals(input.ApprovalsRequired)
	eligibleUserIDs, err := s.normalizeApprovalRuleUserIDs(ctx, projectID, input.EligibleUserIDs)
	if err != nil {
		return gitports.CreateProjectMergeRequestApprovalRuleInput{}, err
	}
	return gitports.CreateProjectMergeRequestApprovalRuleInput{
		ProjectID:         projectID,
		Name:              name,
		TargetBranch:      targetBranch,
		ApprovalsRequired: required,
		EligibleUserIDs:   encodeApprovalRuleUserIDs(eligibleUserIDs),
		CodeOwner:         boolInt(input.CodeOwner),
	}, nil
}

func (s *Service) normalizeUpdateApprovalRuleInput(ctx context.Context, projectID int64, existing mergedomain.ProjectMergeRequestApprovalRule, input UpdateApprovalRuleInput) (gitports.UpdateProjectMergeRequestApprovalRuleInput, error) {
	patch := gitports.UpdateProjectMergeRequestApprovalRuleInput{}
	if err := setApprovalRuleNamePatch(projectID, existing.ID, input.Name, &patch); err != nil {
		return patch, err
	}
	setApprovalRuleTargetBranchPatch(input.TargetBranch, &patch)
	setApprovalRuleRequiredPatch(input.ApprovalsRequired, &patch)
	if err := s.setApprovalRuleEligibleUsersPatch(ctx, projectID, input.EligibleUserIDs, &patch); err != nil {
		return patch, err
	}
	setApprovalRuleCodeOwnerPatch(input.CodeOwner, &patch)
	return patch, nil
}

func setApprovalRuleNamePatch(projectID, ruleID int64, input *string, patch *gitports.UpdateProjectMergeRequestApprovalRuleInput) error {
	if input == nil {
		return nil
	}
	name := strings.TrimSpace(*input)
	if name == "" {
		return apperror.BadRequest("approval rule name is required", oops.In("merge_request").With("project_id", projectID, "rule_id", ruleID).New("approval rule name is required"))
	}
	patch.Name = &name
	return nil
}

func setApprovalRuleTargetBranchPatch(input *string, patch *gitports.UpdateProjectMergeRequestApprovalRuleInput) {
	if input == nil {
		return
	}
	targetBranch := strings.TrimSpace(*input)
	if targetBranch == "" {
		targetBranch = "*"
	}
	patch.TargetBranch = &targetBranch
}

func setApprovalRuleRequiredPatch(input *int, patch *gitports.UpdateProjectMergeRequestApprovalRuleInput) {
	if input == nil {
		return
	}
	required := normalizedRequiredApprovals(*input)
	patch.ApprovalsRequired = &required
}

func (s *Service) setApprovalRuleEligibleUsersPatch(ctx context.Context, projectID int64, input *[]int64, patch *gitports.UpdateProjectMergeRequestApprovalRuleInput) error {
	if input == nil {
		return nil
	}
	eligibleUserIDs, err := s.normalizeApprovalRuleUserIDs(ctx, projectID, *input)
	if err != nil {
		return err
	}
	encoded := encodeApprovalRuleUserIDs(eligibleUserIDs)
	patch.EligibleUserIDs = &encoded
	return nil
}

func setApprovalRuleCodeOwnerPatch(input *bool, patch *gitports.UpdateProjectMergeRequestApprovalRuleInput) {
	if input == nil {
		return
	}
	codeOwner := boolInt(*input)
	patch.CodeOwner = &codeOwner
}

func (s *Service) normalizeApprovalRuleUserIDs(ctx context.Context, projectID int64, userIDs []int64) ([]int64, error) {
	seen := setx.NewOrderedSetWithCapacity[int64](len(userIDs))
	for _, userID := range userIDs {
		if userID <= 0 {
			return nil, apperror.BadRequest("approval rule user id must be positive", oops.In("merge_request").With("project_id", projectID, "user_id", userID).New("approval rule user id must be positive"))
		}
		if seen.Contains(userID) {
			continue
		}
		if _, err := s.userRepo.GetByID(ctx, userID); err != nil {
			return nil, apperror.NotFound("approval rule user not found", oops.In("merge_request").With("project_id", projectID, "user_id", userID).Wrapf(err, "load approval rule user"))
		}
		seen.Add(userID)
	}
	return seen.Values(), nil
}

func approvalRuleView(rule mergedomain.ProjectMergeRequestApprovalRule) (ApprovalRuleView, error) {
	eligibleUserIDs, err := decodeApprovalRuleUserIDs(rule.EligibleUserIDs)
	if err != nil {
		return ApprovalRuleView{}, oops.In("merge_request").With("project_id", rule.ProjectID, "rule_id", rule.ID, "eligible_user_ids", rule.EligibleUserIDs).Wrapf(err, "decode approval rule eligible users")
	}
	return ApprovalRuleView{
		ID:                rule.ID,
		ProjectID:         rule.ProjectID,
		Name:              rule.Name,
		TargetBranch:      rule.TargetBranch,
		ApprovalsRequired: normalizedRequiredApprovals(rule.ApprovalsRequired),
		EligibleUserIDs:   eligibleUserIDs,
		CodeOwner:         rule.RequiresCodeOwner(),
	}, nil
}

func encodeApprovalRuleUserIDs(values []int64) string {
	parts := make([]string, 0, len(values))
	for _, value := range values {
		if value > 0 {
			parts = append(parts, strconv.FormatInt(value, 10))
		}
	}
	return strings.Join(parts, ",")
}

func decodeApprovalRuleUserIDs(value string) ([]int64, error) {
	trimmed := strings.TrimSpace(value)
	if trimmed == "" {
		return []int64{}, nil
	}
	parts := strings.FieldsFunc(trimmed, func(r rune) bool {
		return r == ',' || r == ' ' || r == '\n' || r == '\t'
	})
	ids := make([]int64, 0, len(parts))
	for _, part := range parts {
		id, err := strconv.ParseInt(strings.TrimSpace(part), 10, 64)
		if err != nil {
			return nil, oops.In("merge_request").With("value", part).Wrapf(err, "parse approval rule user id")
		}
		if id > 0 {
			ids = append(ids, id)
		}
	}
	return ids, nil
}

func boolInt(value bool) int {
	if value {
		return 1
	}
	return 0
}
