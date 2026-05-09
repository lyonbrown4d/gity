package mergerequest

import (
	"context"
	"strings"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	mergedomain "github.com/DaiYuANg/gity/internal/domain/merge"
	"github.com/samber/oops"
)

type CommentInput struct {
	AuthorUserID int64  `json:"author_user_id"`
	Body         string `json:"body"`
}

type CommentsView struct {
	MergeRequest mergedomain.ProjectMergeRequest          `json:"merge_request"`
	Comments     []mergedomain.ProjectMergeRequestComment `json:"comments"`
}

type ApprovalInput struct {
	UserID int64 `json:"user_id"`
}

type ApprovalsView struct {
	MergeRequest mergedomain.ProjectMergeRequest           `json:"merge_request"`
	Approvals    []mergedomain.ProjectMergeRequestApproval `json:"approvals"`
}

func (s *Service) ListComments(ctx context.Context, projectID, mergeIID int64) (CommentsView, error) {
	mr, err := s.loadMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return CommentsView{}, err
	}
	comments, err := s.listCommentsByMergeRequestID(ctx, mr.ID)
	if err != nil {
		return CommentsView{}, err
	}
	return CommentsView{MergeRequest: mr, Comments: comments}, nil
}

func (s *Service) CreateComment(ctx context.Context, projectID, mergeIID int64, input CommentInput) (CommentsView, error) {
	mr, err := s.loadMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return CommentsView{}, err
	}
	body := strings.TrimSpace(input.Body)
	if body == "" {
		bodyErr := oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID, "author_user_id", input.AuthorUserID).New("merge request comment body is required")
		return CommentsView{}, apperror.BadRequest("merge request comment body is required", bodyErr)
	}
	if userErr := s.ensureCollaborationUser(ctx, projectID, mergeIID, input.AuthorUserID, "comment author"); userErr != nil {
		return CommentsView{}, userErr
	}
	if s.commentRepo == nil {
		return CommentsView{}, oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID).New("merge request comment repository is not configured")
	}
	if _, createErr := s.commentRepo.Create(ctx, gitports.CreateProjectMergeRequestCommentInput{
		MergeRequestID: mr.ID,
		AuthorUserID:   input.AuthorUserID,
		Body:           body,
	}); createErr != nil {
		return CommentsView{}, oops.In("merge_request").With("project_id", projectID, "merge_request_id", mr.ID, "merge_iid", mergeIID, "author_user_id", input.AuthorUserID).Wrapf(createErr, "create merge request comment")
	}
	comments, err := s.listCommentsByMergeRequestID(ctx, mr.ID)
	if err != nil {
		return CommentsView{}, err
	}
	return CommentsView{MergeRequest: mr, Comments: comments}, nil
}

func (s *Service) ListApprovals(ctx context.Context, projectID, mergeIID int64) (ApprovalsView, error) {
	mr, err := s.loadMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return ApprovalsView{}, err
	}
	approvals, err := s.listApprovalsByMergeRequestID(ctx, mr.ID)
	if err != nil {
		return ApprovalsView{}, err
	}
	return ApprovalsView{MergeRequest: mr, Approvals: approvals}, nil
}

func (s *Service) Approve(ctx context.Context, projectID, mergeIID int64, input ApprovalInput) (ApprovalsView, error) {
	mr, err := s.loadMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return ApprovalsView{}, err
	}
	if stateErr := ensureMergeRequestOpen(projectID, mergeIID, mr); stateErr != nil {
		return ApprovalsView{}, stateErr
	}
	if userErr := s.ensureCollaborationUser(ctx, projectID, mergeIID, input.UserID, "approval user"); userErr != nil {
		return ApprovalsView{}, userErr
	}
	if s.approvalRepo == nil {
		return ApprovalsView{}, oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID).New("merge request approval repository is not configured")
	}
	if _, approveErr := s.approvalRepo.Upsert(ctx, gitports.UpsertProjectMergeRequestApprovalInput{MergeRequestID: mr.ID, UserID: input.UserID}); approveErr != nil {
		return ApprovalsView{}, oops.In("merge_request").With("project_id", projectID, "merge_request_id", mr.ID, "merge_iid", mergeIID, "user_id", input.UserID).Wrapf(approveErr, "approve merge request")
	}
	approvals, err := s.listApprovalsByMergeRequestID(ctx, mr.ID)
	if err != nil {
		return ApprovalsView{}, err
	}
	return ApprovalsView{MergeRequest: mr, Approvals: approvals}, nil
}

func (s *Service) Unapprove(ctx context.Context, projectID, mergeIID int64, input ApprovalInput) (ApprovalsView, error) {
	mr, err := s.loadMergeRequest(ctx, projectID, mergeIID)
	if err != nil {
		return ApprovalsView{}, err
	}
	if stateErr := ensureMergeRequestOpen(projectID, mergeIID, mr); stateErr != nil {
		return ApprovalsView{}, stateErr
	}
	if userErr := s.ensureCollaborationUser(ctx, projectID, mergeIID, input.UserID, "approval user"); userErr != nil {
		return ApprovalsView{}, userErr
	}
	if s.approvalRepo == nil {
		return ApprovalsView{}, oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID).New("merge request approval repository is not configured")
	}
	if deleteErr := s.approvalRepo.DeleteByMergeRequestAndUser(ctx, mr.ID, input.UserID); deleteErr != nil {
		return ApprovalsView{}, oops.In("merge_request").With("project_id", projectID, "merge_request_id", mr.ID, "merge_iid", mergeIID, "user_id", input.UserID).Wrapf(deleteErr, "unapprove merge request")
	}
	approvals, err := s.listApprovalsByMergeRequestID(ctx, mr.ID)
	if err != nil {
		return ApprovalsView{}, err
	}
	return ApprovalsView{MergeRequest: mr, Approvals: approvals}, nil
}

func (s *Service) listCommentsByMergeRequestID(ctx context.Context, mergeRequestID int64) ([]mergedomain.ProjectMergeRequestComment, error) {
	if s.commentRepo == nil {
		return nil, oops.In("merge_request").With("merge_request_id", mergeRequestID).New("merge request comment repository is not configured")
	}
	comments, err := s.commentRepo.ListByMergeRequestID(ctx, mergeRequestID)
	if err != nil {
		return nil, oops.In("merge_request").With("merge_request_id", mergeRequestID).Wrapf(err, "list merge request comments")
	}
	return comments.Values(), nil
}

func (s *Service) listApprovalsByMergeRequestID(ctx context.Context, mergeRequestID int64) ([]mergedomain.ProjectMergeRequestApproval, error) {
	if s.approvalRepo == nil {
		return nil, oops.In("merge_request").With("merge_request_id", mergeRequestID).New("merge request approval repository is not configured")
	}
	approvals, err := s.approvalRepo.ListByMergeRequestID(ctx, mergeRequestID)
	if err != nil {
		return nil, oops.In("merge_request").With("merge_request_id", mergeRequestID).Wrapf(err, "list merge request approvals")
	}
	return approvals.Values(), nil
}

func (s *Service) ensureCollaborationUser(ctx context.Context, projectID, mergeIID, userID int64, label string) error {
	if userID <= 0 {
		err := oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID, "user_id", userID).New(label + " id must be positive")
		return apperror.BadRequest(label+" id must be positive", err)
	}
	if _, err := s.userRepo.GetByID(ctx, userID); err != nil {
		wrapped := oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID, "user_id", userID, "collaboration_user", label).Wrapf(err, "load collaboration user")
		return apperror.NotFound("merge request "+label+" not found", wrapped)
	}
	return nil
}

func ensureMergeRequestOpen(projectID, mergeIID int64, mr mergedomain.ProjectMergeRequest) error {
	if mr.State == "opened" {
		return nil
	}
	err := oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID, "state", mr.State).New("merge request state is " + mr.State)
	return apperror.Conflict("merge request is not opened", err)
}
