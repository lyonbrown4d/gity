package issue

import (
	"context"
	"errors"
	"log/slog"
	"strings"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	storageports "github.com/DaiYuANg/gity/internal/application/ports"
	issuedomain "github.com/DaiYuANg/gity/internal/domain/issue"
	"github.com/samber/oops"
)

type Service struct {
	logger         *slog.Logger
	projectRepo    storageports.ProjectRepository
	issueRepo      storageports.ProjectIssueRepository
	commentRepo    storageports.ProjectIssueCommentRepository
	attachmentRepo storageports.ProjectIssueAttachmentRepository
	assigneeRepo   storageports.ProjectIssueAssigneeRepository
	labelRepo      storageports.ProjectIssueLabelRepository
	userRepo       storageports.UserRepository
	storage        storageports.ObjectStorage
}

type Repositories struct {
	projectRepo    storageports.ProjectRepository
	issueRepo      storageports.ProjectIssueRepository
	commentRepo    storageports.ProjectIssueCommentRepository
	attachmentRepo storageports.ProjectIssueAttachmentRepository
	assigneeRepo   storageports.ProjectIssueAssigneeRepository
	labelRepo      storageports.ProjectIssueLabelRepository
}

type RuntimeDependencies struct {
	logger   *slog.Logger
	userRepo storageports.UserRepository
	storage  storageports.ObjectStorage
}

type CreateIssueInput struct {
	AuthorUserID int64  `json:"author_user_id"`
	Title        string `json:"title"`
	Description  string `json:"description"`
}

type UpdateIssueInput struct {
	Title       *string `json:"title"`
	Description *string `json:"description"`
	State       *string `json:"state"`
}

type CreateCommentInput struct {
	AuthorUserID int64  `json:"author_user_id"`
	Body         string `json:"body"`
}

func NewRepositories(projectRepo storageports.ProjectRepository, issueRepo storageports.ProjectIssueRepository, commentRepo storageports.ProjectIssueCommentRepository, attachmentRepo storageports.ProjectIssueAttachmentRepository, assigneeRepo storageports.ProjectIssueAssigneeRepository, labelRepo storageports.ProjectIssueLabelRepository) Repositories {
	return Repositories{
		projectRepo:    projectRepo,
		issueRepo:      issueRepo,
		commentRepo:    commentRepo,
		attachmentRepo: attachmentRepo,
		assigneeRepo:   assigneeRepo,
		labelRepo:      labelRepo,
	}
}

func NewRuntimeDependencies(logger *slog.Logger, userRepo storageports.UserRepository, storage storageports.ObjectStorage) RuntimeDependencies {
	return RuntimeDependencies{logger: logger, userRepo: userRepo, storage: storage}
}

func NewService(logger *slog.Logger, projectRepo storageports.ProjectRepository, issueRepo storageports.ProjectIssueRepository, commentRepo storageports.ProjectIssueCommentRepository, attachmentRepo storageports.ProjectIssueAttachmentRepository, userRepo storageports.UserRepository, storage storageports.ObjectStorage) *Service {
	return NewServiceWithDependencies(
		NewRepositories(projectRepo, issueRepo, commentRepo, attachmentRepo, nil, nil),
		NewRuntimeDependencies(logger, userRepo, storage),
	)
}

func NewServiceWithDependencies(repos Repositories, runtime RuntimeDependencies) *Service {
	return &Service{
		logger:         runtime.logger,
		projectRepo:    repos.projectRepo,
		issueRepo:      repos.issueRepo,
		commentRepo:    repos.commentRepo,
		attachmentRepo: repos.attachmentRepo,
		assigneeRepo:   repos.assigneeRepo,
		labelRepo:      repos.labelRepo,
		userRepo:       runtime.userRepo,
		storage:        runtime.storage,
	}
}

func (s *Service) ListIssues(ctx context.Context, projectID int64) ([]issuedomain.ProjectIssue, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	items, err := s.issueRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, oops.In("issue").With("project_id", projectID).Wrapf(err, "list project issues")
	}
	return items.Values(), nil
}

func (s *Service) GetIssueByIID(ctx context.Context, projectID, issueIID int64) (issuedomain.ProjectIssue, error) {
	return s.loadIssue(ctx, projectID, issueIID)
}

func (s *Service) CreateIssue(ctx context.Context, projectID int64, input CreateIssueInput) (issuedomain.ProjectIssue, error) {
	if strings.TrimSpace(input.Title) == "" {
		return issuedomain.ProjectIssue{}, oops.In("issue").With("project_id", projectID, "author_user_id", input.AuthorUserID).New("issue title is required")
	}
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return issuedomain.ProjectIssue{}, apperror.NotFound("project not found", err)
	}
	if _, err := s.userRepo.GetByID(ctx, input.AuthorUserID); err != nil {
		return issuedomain.ProjectIssue{}, apperror.NotFound("issue author not found", err)
	}
	issue, err := s.issueRepo.Create(ctx, storageports.CreateProjectIssueInput{
		ProjectID:    projectID,
		AuthorUserID: input.AuthorUserID,
		Title:        input.Title,
		Description:  input.Description,
	})
	if err != nil {
		return issuedomain.ProjectIssue{}, oops.In("issue").With("project_id", projectID, "author_user_id", input.AuthorUserID).Wrapf(err, "create issue")
	}
	return issue, nil
}

func (s *Service) UpdateIssue(ctx context.Context, projectID, issueIID int64, input UpdateIssueInput) (issuedomain.ProjectIssue, error) {
	issue, err := s.loadIssue(ctx, projectID, issueIID)
	if err != nil {
		return issuedomain.ProjectIssue{}, err
	}
	if input.Title != nil && strings.TrimSpace(*input.Title) == "" {
		return issuedomain.ProjectIssue{}, oops.In("issue").With("project_id", projectID, "issue_iid", issueIID).New("issue title is required")
	}
	if input.State != nil {
		state := strings.TrimSpace(*input.State)
		if state != "opened" && state != "closed" {
			return issuedomain.ProjectIssue{}, oops.In("issue").With("project_id", projectID, "issue_iid", issueIID, "state", state).New("issue state must be opened or closed")
		}
	}
	if err := s.issueRepo.UpdateByID(ctx, issue.ID, storageports.UpdateProjectIssueInput{Title: input.Title, Description: input.Description, State: input.State}); err != nil {
		return issuedomain.ProjectIssue{}, oops.In("issue").With("project_id", projectID, "issue_id", issue.ID, "issue_iid", issueIID).Wrapf(err, "update issue")
	}
	return s.loadIssue(ctx, projectID, issueIID)
}

func (s *Service) ListComments(ctx context.Context, projectID, issueIID int64) ([]issuedomain.ProjectIssueComment, error) {
	issue, err := s.loadIssue(ctx, projectID, issueIID)
	if err != nil {
		return nil, err
	}
	items, err := s.commentRepo.ListByIssueID(ctx, issue.ID)
	if err != nil {
		return nil, oops.In("issue").With("project_id", projectID, "issue_id", issue.ID, "issue_iid", issueIID).Wrapf(err, "list issue comments")
	}
	return items.Values(), nil
}

func (s *Service) CreateComment(ctx context.Context, projectID, issueIID int64, input CreateCommentInput) (issuedomain.ProjectIssueComment, error) {
	issue, err := s.loadIssue(ctx, projectID, issueIID)
	if err != nil {
		return issuedomain.ProjectIssueComment{}, err
	}
	if strings.TrimSpace(input.Body) == "" {
		return issuedomain.ProjectIssueComment{}, oops.In("issue").With("project_id", projectID, "issue_iid", issueIID, "author_user_id", input.AuthorUserID).New("issue comment body is required")
	}
	if _, authorErr := s.userRepo.GetByID(ctx, input.AuthorUserID); authorErr != nil {
		return issuedomain.ProjectIssueComment{}, apperror.NotFound("comment author not found", authorErr)
	}
	comment, err := s.commentRepo.Create(ctx, storageports.CreateProjectIssueCommentInput{ProjectIssueID: issue.ID, AuthorUserID: input.AuthorUserID, Body: input.Body})
	if err != nil {
		return issuedomain.ProjectIssueComment{}, oops.In("issue").With("project_id", projectID, "issue_id", issue.ID, "issue_iid", issueIID, "author_user_id", input.AuthorUserID).Wrapf(err, "create issue comment")
	}
	return comment, nil
}

func (s *Service) loadIssue(ctx context.Context, projectID, issueIID int64) (issuedomain.ProjectIssue, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return issuedomain.ProjectIssue{}, apperror.NotFound("project not found", err)
	}
	issue, err := s.issueRepo.GetByProjectAndIID(ctx, projectID, issueIID)
	if err != nil {
		if errors.Is(err, storageports.ErrNotFound) {
			return issuedomain.ProjectIssue{}, apperror.NotFound("issue not found", err)
		}
		return issuedomain.ProjectIssue{}, oops.In("issue").With("project_id", projectID, "issue_iid", issueIID).Wrapf(err, "load issue by iid")
	}
	return issue, nil
}
