package project

import (
	"context"
	"errors"
	"fmt"
	"strings"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/samber/oops"
)

func (s *Service) ListBranches(ctx context.Context, id int64) ([]Branch, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	gitBranches, err := s.gitRepository.ListBranches(ctx, repositoryPath(project), project.DefaultBranch)
	if err != nil {
		return nil, mapGitError(err)
	}
	protections, err := s.branchRepo.ListByProjectID(ctx, id)
	if err != nil {
		return nil, oops.In("project").With("project_id", id).Wrapf(err, "list protected branches")
	}
	return collectionx.MapList(collectionx.NewList(gitBranches...), func(_ int, branch gitports.Branch) Branch {
		protection := matchingProtectionView(protections, branch.Name)
		return Branch{
			Name:          branch.Name,
			Hash:          branch.Hash,
			IsDefault:     branch.IsDefault,
			IsProtected:   protection != nil,
			LastCommitSHA: branch.Hash,
			Protection:    protection,
		}
	}).Values(), nil
}

func (s *Service) CreateBranch(ctx context.Context, id int64, branchName, sourceRef string) (Branch, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return Branch{}, apperror.NotFound("project not found", err)
	}
	branchName = strings.TrimSpace(branchName)
	if branchName == "" {
		return Branch{}, errors.New("branch name is required")
	}
	if strings.TrimSpace(sourceRef) == "" {
		sourceRef = project.DefaultBranch
	}
	if err := s.ensureBranchAllowsDirectPush(ctx, id, branchName); err != nil {
		return Branch{}, err
	}
	if err := s.gitRunner.CreateBranch(ctx, repositoryPath(project), branchName, sourceRef); err != nil {
		return Branch{}, mapGitExecError(err)
	}
	return s.GetBranch(ctx, id, branchName)
}

func (s *Service) SetBranchProtection(ctx context.Context, id int64, branchName string, protected bool) (Branch, error) {
	if _, err := s.GetBranch(ctx, id, branchName); err != nil {
		return Branch{}, err
	}
	if protected {
		protection, err := s.branchRepo.Protect(ctx, id, branchName)
		if err != nil {
			return Branch{}, oops.In("project").With("project_id", id, "branch", branchName).Wrapf(err, "protect branch")
		}
		s.publishProjectEventAsync(ctx, id, projectdomain.NewProjectBranchProtectionChangedEvent(protection, true))
	} else if err := s.branchRepo.Unprotect(ctx, id, branchName); err != nil {
		return Branch{}, oops.In("project").With("project_id", id, "branch", branchName).Wrapf(err, "unprotect branch")
	} else {
		s.publishProjectEventAsync(ctx, id, projectdomain.NewProjectBranchUnprotectedEvent(id, branchName))
	}
	return s.GetBranch(ctx, id, branchName)
}

func (s *Service) ListBranchProtections(ctx context.Context, id int64) ([]BranchProtection, error) {
	if _, err := s.repo.GetByID(ctx, id); err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	items, err := s.branchRepo.ListByProjectID(ctx, id)
	if err != nil {
		return nil, oops.In("project").With("project_id", id).Wrapf(err, "list branch protections")
	}
	return collectionx.MapList(items, func(_ int, item projectdomain.ProjectBranchProtection) BranchProtection {
		return toBranchProtection(item)
	}).Values(), nil
}

func (s *Service) UpsertBranchProtection(ctx context.Context, id int64, input BranchProtectionInput) (BranchProtection, error) {
	if _, err := s.repo.GetByID(ctx, id); err != nil {
		return BranchProtection{}, apperror.NotFound("project not found", err)
	}
	branchName := strings.TrimSpace(input.BranchName)
	if branchName == "" {
		return BranchProtection{}, apperror.BadRequest("branch name is required", oops.In("project").With("project_id", id).New("branch name is required"))
	}
	item, err := s.branchRepo.Upsert(ctx, id, gitports.UpsertProjectBranchProtectionInput{
		BranchName:             branchName,
		RuleType:               input.RuleType,
		PushAccessLevel:        input.PushAccessLevel,
		MergeAccessLevel:       input.MergeAccessLevel,
		RequireMergeRequest:    input.RequireMergeRequest,
		RequirePipelineSuccess: input.RequirePipelineSuccess,
		AllowForcePush:         input.AllowForcePush,
		AllowDelete:            input.AllowDelete,
	})
	if err != nil {
		return BranchProtection{}, oops.In("project").With("project_id", id, "branch", branchName).Wrapf(err, "upsert branch protection")
	}
	s.publishProjectEventAsync(ctx, id, projectdomain.NewProjectBranchProtectionChangedEvent(item, true))
	return toBranchProtection(item), nil
}

func (s *Service) DeleteBranch(ctx context.Context, id int64, branchName string) error {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return apperror.NotFound("project not found", err)
	}
	branchName = strings.TrimSpace(branchName)
	if branchName == "" {
		return apperror.BadRequest("branch name is required", oops.In("project").With("project_id", id).New("branch name is required"))
	}
	if branchName == project.DefaultBranch {
		return apperror.BadRequest("default branch cannot be deleted", oops.In("project").With("project_id", id, "branch", branchName).New("default branch cannot be deleted"))
	}
	protection, protected, err := s.branchProtectionFor(ctx, id, branchName)
	if err != nil {
		return err
	}
	if protected && protection.BlocksDelete() {
		return apperror.Forbidden("protected branch cannot be deleted", fmt.Errorf("branch is protected: %s", branchName))
	}
	if err := s.gitRunner.DeleteBranch(ctx, repositoryPath(project), branchName); err != nil {
		return mapGitExecError(err)
	}
	s.publishProjectEventAsync(ctx, id, projectdomain.NewProjectBranchDeletedEvent(id, branchName))
	return nil
}

func (s *Service) GetBranch(ctx context.Context, id int64, branchName string) (Branch, error) {
	branchName = strings.TrimSpace(branchName)
	if branchName == "" {
		return Branch{}, errors.New("branch name is required")
	}
	branches, err := s.ListBranches(ctx, id)
	if err != nil {
		return Branch{}, err
	}
	for _, branch := range branches {
		if branch.Name == branchName {
			return branch, nil
		}
	}
	return Branch{}, apperror.NotFound("branch not found", gitports.ErrReferenceNotFound)
}

func (s *Service) CreateFileCommit(ctx context.Context, id int64, input CreateFileCommitInput) error {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return apperror.NotFound("project not found", err)
	}
	branchName := strings.TrimSpace(input.BranchName)
	if branchName == "" {
		branchName = project.DefaultBranch
	}
	if allowErr := s.ensureBranchAllowsDirectPush(ctx, id, branchName); allowErr != nil {
		return allowErr
	}
	err = s.gitRunner.CreateFileCommit(ctx, repositoryPath(project), gitports.CreateFileCommitInput{
		BranchName:  branchName,
		FilePath:    input.Path,
		Content:     input.Content,
		Message:     input.Message,
		AuthorName:  input.AuthorName,
		AuthorEmail: input.AuthorEmail,
	})
	if err != nil {
		return mapGitExecError(err)
	}
	return nil
}

func (s *Service) ensureBranchAllowsDirectPush(ctx context.Context, projectID int64, branchName string) error {
	protection, protected, err := s.branchProtectionFor(ctx, projectID, branchName)
	if err != nil {
		return err
	}
	if protected && protection.BlocksDirectPush() {
		return apperror.Forbidden("protected branch cannot be updated", fmt.Errorf("branch is protected: %s", branchName))
	}
	return nil
}

func (s *Service) branchProtectionFor(ctx context.Context, projectID int64, branchName string) (projectdomain.ProjectBranchProtection, bool, error) {
	item, err := s.branchRepo.MatchByProjectAndBranch(ctx, projectID, branchName)
	if err == nil {
		return item, true, nil
	}
	if errors.Is(err, gitports.ErrNotFound) {
		return projectdomain.ProjectBranchProtection{}, false, nil
	}
	return projectdomain.ProjectBranchProtection{}, false, oops.In("project").With("project_id", projectID, "branch", branchName).Wrapf(err, "check branch protection")
}

func matchingProtectionView(items *collectionx.List[projectdomain.ProjectBranchProtection], branchName string) *BranchProtection {
	if items == nil {
		return nil
	}
	var patternMatch BranchProtection
	patternMatched := false
	values := items.Values()
	for index := range values {
		item := &values[index]
		if !item.MatchesBranch(branchName) {
			continue
		}
		view := toBranchProtection(*item)
		if view.RuleType == projectdomain.ProjectBranchProtectionRuleExact {
			return &view
		}
		if !patternMatched {
			patternMatch = view
			patternMatched = true
		}
	}
	if patternMatched {
		return &patternMatch
	}
	return nil
}

func toBranchProtection(item projectdomain.ProjectBranchProtection) BranchProtection {
	return BranchProtection{
		ID:                     item.ID,
		ProjectID:              item.ProjectID,
		BranchName:             item.BranchName,
		RuleType:               projectdomain.NormalizeProjectBranchProtectionRuleType(item.RuleType, item.BranchName),
		PushAccessLevel:        projectdomain.NormalizeProjectBranchProtectionAccessLevel(item.PushAccessLevel, projectdomain.ProjectBranchProtectionAccessNoOne),
		MergeAccessLevel:       projectdomain.NormalizeProjectBranchProtectionAccessLevel(item.MergeAccessLevel, projectdomain.ProjectBranchProtectionAccessMaintainer),
		RequireMergeRequest:    item.RequireMergeRequest != 0,
		RequirePipelineSuccess: item.RequirePipelineSuccess != 0,
		AllowForcePush:         item.AllowForcePush != 0,
		AllowDelete:            item.AllowDelete != 0,
	}
}
