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
	setx "github.com/arcgolabs/collectionx/set"
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
	protected, err := s.protectedBranchSet(ctx, id)
	if err != nil {
		return nil, err
	}
	return collectionx.MapList(collectionx.NewList(gitBranches...), func(_ int, branch gitports.Branch) Branch {
		return Branch{
			Name:          branch.Name,
			Hash:          branch.Hash,
			IsDefault:     branch.IsDefault,
			IsProtected:   protected.Contains(branch.Name),
			LastCommitSHA: branch.Hash,
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
		if _, err := s.branchRepo.Protect(ctx, id, branchName); err != nil {
			return Branch{}, oops.In("project").With("project_id", id, "branch", branchName).Wrapf(err, "protect branch")
		}
	} else if err := s.branchRepo.Unprotect(ctx, id, branchName); err != nil {
		return Branch{}, oops.In("project").With("project_id", id, "branch", branchName).Wrapf(err, "unprotect branch")
	}
	return s.GetBranch(ctx, id, branchName)
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
	protected, err := s.isBranchProtected(ctx, id, branchName)
	if err != nil {
		return err
	}
	if protected {
		return apperror.Forbidden("protected branch cannot be updated", fmt.Errorf("branch is protected: %s", branchName))
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

func (s *Service) isBranchProtected(ctx context.Context, projectID int64, branchName string) (bool, error) {
	if _, err := s.branchRepo.GetByProjectAndBranch(ctx, projectID, branchName); err == nil {
		return true, nil
	} else if !errors.Is(err, gitports.ErrNotFound) {
		return false, oops.In("project").With("project_id", projectID, "branch", branchName).Wrapf(err, "check branch protection")
	}
	return false, nil
}

func (s *Service) protectedBranchSet(ctx context.Context, projectID int64) (*setx.Set[string], error) {
	items, err := s.branchRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, oops.In("project").With("project_id", projectID).Wrapf(err, "list protected branches")
	}
	return setx.NewSetWithCapacity[string](items.Len(), collectionx.MapList(items, func(_ int, item projectdomain.ProjectBranchProtection) string {
		return item.BranchName
	}).Values()...), nil
}
