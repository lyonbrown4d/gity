package mergerequest

import (
	"context"
	"errors"
	"fmt"
	"strings"

	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	"github.com/samber/oops"
)

func (s *Service) Merge(ctx context.Context, projectID, mergeIID int64, input MergeInput) (mergedomain.ProjectMergeRequest, error) {
	project, mr, err := s.loadProjectMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return mergedomain.ProjectMergeRequest{}, err
	}
	if readyErr := s.ensureMergeReady(ctx, projectID, mergeIID, project, mr, input.ActorUserID); readyErr != nil {
		return mergedomain.ProjectMergeRequest{}, readyErr
	}
	if mergeErr := s.gitRunner.MergeBranches(ctx, project.FullPath+".git", mergeBranchesInput(mr, input)); mergeErr != nil {
		return mergedomain.ProjectMergeRequest{}, mapGitExecError(mergeErr)
	}
	if markErr := s.markMerged(ctx, projectID, mergeIID, mr.ID); markErr != nil {
		return mergedomain.ProjectMergeRequest{}, markErr
	}
	s.triggerTargetBranchPipeline(ctx, project, mr)
	mergedMR, err := s.loadMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return mergedomain.ProjectMergeRequest{}, err
	}
	s.publishRepositoryChanged(ctx, project, mr.TargetBranch)
	s.publishEventAsync(ctx, mergedomain.NewProjectMergeRequestMergedEvent(mergedMR, input.ActorUserID))
	return mergedMR, nil
}

func (s *Service) ensureMergeReady(ctx context.Context, projectID, mergeIID int64, project projectdomain.Project, mr mergedomain.ProjectMergeRequest, actorUserID int64) error {
	if mr.State != "opened" {
		return apperror.Conflict("merge request is not opened", fmt.Errorf("merge request state: %s", mr.State))
	}
	if err := s.ensureTargetBranchMergeAccess(ctx, projectID, mergeIID, project, mr, actorUserID); err != nil {
		return err
	}
	checks, err := s.evaluateChecks(ctx, project, mr)
	if err != nil {
		return err
	}
	if checks.Required && !checks.Mergeable {
		return apperror.Conflict("merge request checks are not satisfied", errors.New(checks.BlockingReason))
	}
	return nil
}

func mergeBranchesInput(mr mergedomain.ProjectMergeRequest, input MergeInput) gitports.MergeBranchesInput {
	message := strings.TrimSpace(input.Message)
	if message == "" {
		message = fmt.Sprintf("Merge branch '%s' into '%s'", mr.SourceBranch, mr.TargetBranch)
	}
	return gitports.MergeBranchesInput{
		TargetBranch: mr.TargetBranch,
		SourceBranch: mr.SourceBranch,
		Message:      message,
		AuthorName:   input.AuthorName,
		AuthorEmail:  input.AuthorEmail,
	}
}

func (s *Service) markMerged(ctx context.Context, projectID, mergeIID, mergeRequestID int64) error {
	merged := "merged"
	if err := s.mergeRepo.UpdateByID(ctx, mergeRequestID, gitports.UpdateProjectMergeRequestInput{State: &merged}); err != nil {
		return oops.In("merge_request").With("project_id", projectID, "merge_request_id", mergeRequestID, "merge_iid", mergeIID).Wrapf(err, "mark merge request merged")
	}
	return nil
}
