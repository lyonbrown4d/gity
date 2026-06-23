package mergerequest

import (
	"context"
	"encoding/base64"
	"errors"
	"fmt"

	setx "github.com/arcgolabs/collectionx/set"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	"github.com/samber/oops"
)

type codeOwnerIDCache struct {
	loaded bool
	ids    []int64
}

func (s *Service) approvalRuleCheck(ctx context.Context, project projectdomain.Project, mr mergedomain.ProjectMergeRequest, rule mergedomain.ProjectMergeRequestApprovalRule, approvers *setx.Set[int64], cache *codeOwnerIDCache) (ApprovalRuleCheck, bool, error) {
	eligibleUserIDs, err := s.approvalRuleEligibleUserIDs(ctx, project, mr, rule, cache)
	if err != nil {
		return ApprovalRuleCheck{}, false, err
	}
	if rule.RequiresCodeOwner() && len(eligibleUserIDs) == 0 {
		return ApprovalRuleCheck{}, false, nil
	}
	check := ApprovalRuleCheck{
		RuleID:            rule.ID,
		Name:              rule.Name,
		TargetBranch:      rule.TargetBranch,
		ApprovalsRequired: normalizedRequiredApprovals(rule.ApprovalsRequired),
		EligibleUserIDs:   eligibleUserIDs,
		CodeOwner:         rule.RequiresCodeOwner(),
	}
	return evaluateApprovalRuleCheck(check, approvers), true, nil
}

func (s *Service) approvalRuleEligibleUserIDs(ctx context.Context, project projectdomain.Project, mr mergedomain.ProjectMergeRequest, rule mergedomain.ProjectMergeRequestApprovalRule, cache *codeOwnerIDCache) ([]int64, error) {
	eligibleUserIDs, err := decodeApprovalRuleUserIDs(rule.EligibleUserIDs)
	if err != nil {
		return nil, oops.In("merge_request").With("project_id", project.ID, "rule_id", rule.ID).Wrapf(err, "decode approval rule eligible users")
	}
	if !rule.RequiresCodeOwner() {
		return eligibleUserIDs, nil
	}
	if cache == nil {
		cache = &codeOwnerIDCache{}
	}
	if !cache.loaded {
		cache.loaded = true
		cache.ids, err = s.resolveCodeOwnerIDs(ctx, project, mr)
		if err != nil {
			return nil, err
		}
	}
	return cache.ids, nil
}

func (s *Service) loadCodeOwners(ctx context.Context, project projectdomain.Project, targetBranch string) ([]CodeOwnerRule, error) {
	for _, codeOwnersPath := range []string{"CODEOWNERS", ".gitlab/CODEOWNERS", "docs/CODEOWNERS"} {
		blob, err := s.gitRepo.GetBlob(ctx, project.FullPath+".git", targetBranch, project.DefaultBranch, codeOwnersPath)
		if err == nil {
			content, contentErr := codeOwnersBlobContent(blob)
			if contentErr != nil {
				return nil, oops.In("merge_request").With("project_id", project.ID, "path", codeOwnersPath).Wrapf(contentErr, "read CODEOWNERS")
			}
			return ParseCodeOwners(content), nil
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
			return "", oops.In("merge_request").Wrapf(err, "decode CODEOWNERS blob")
		}
		return string(data), nil
	}
	return "", fmt.Errorf("unsupported CODEOWNERS encoding %s", blob.Encoding)
}
