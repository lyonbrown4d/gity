package issue

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"encoding/hex"
	"errors"
	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	storageports "github.com/DaiYuANg/gity/internal/application/ports"
	issuedomain "github.com/DaiYuANg/gity/internal/domain/issue"
	"github.com/samber/oops"
	"log/slog"
	"strings"
)

type Service struct {
	logger         *slog.Logger
	projectRepo    storageports.ProjectRepository
	issueRepo      storageports.ProjectIssueRepository
	commentRepo    storageports.ProjectIssueCommentRepository
	attachmentRepo storageports.ProjectIssueAttachmentRepository
	userRepo       storageports.UserRepository
	storage        storageports.ObjectStorage
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

type CreateAttachmentInput struct {
	UploadedByUserID int64  `json:"uploaded_by_user_id"`
	FileName         string `json:"file_name"`
	ContentType      string `json:"content_type"`
	ContentBase64    string `json:"content_base64"`
}

type AttachmentContentView struct {
	Attachment issuedomain.ProjectIssueAttachment `json:"attachment"`
	Content    string                             `json:"content_base64"`
}

type AttachmentUploadInput struct {
	UploadedByUserID int64
	IssueIID         int64
	FileName         string
	ContentType      string
	Content          []byte
}

type AttachmentUploadView struct {
	Attachment  *issuedomain.ProjectIssueAttachment
	ObjectKey   string
	FileName    string
	ContentType string
	ByteSize    int64
}

type AttachmentRawContent struct {
	FileName    string
	ContentType string
	Content     []byte
}

func NewService(projectRepo storageports.ProjectRepository, issueRepo storageports.ProjectIssueRepository, commentRepo storageports.ProjectIssueCommentRepository, attachmentRepo storageports.ProjectIssueAttachmentRepository, userRepo storageports.UserRepository, storage storageports.ObjectStorage) *Service {
	return &Service{
		logger:         slog.Default(),
		projectRepo:    projectRepo,
		issueRepo:      issueRepo,
		commentRepo:    commentRepo,
		attachmentRepo: attachmentRepo,
		userRepo:       userRepo,
		storage:        storage,
	}
}

func (s *Service) ListIssues(ctx context.Context, projectID int64) ([]issuedomain.ProjectIssue, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	items, err := s.issueRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, err
	}
	return items.Values(), nil
}

func (s *Service) GetIssueByIID(ctx context.Context, projectID int64, issueIID int64) (issuedomain.ProjectIssue, error) {
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
	return s.issueRepo.Create(ctx, storageports.CreateProjectIssueInput{
		ProjectID:    projectID,
		AuthorUserID: input.AuthorUserID,
		Title:        input.Title,
		Description:  input.Description,
	})
}

func (s *Service) UpdateIssue(ctx context.Context, projectID int64, issueIID int64, input UpdateIssueInput) (issuedomain.ProjectIssue, error) {
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
		return issuedomain.ProjectIssue{}, err
	}
	return s.loadIssue(ctx, projectID, issueIID)
}

func (s *Service) ListComments(ctx context.Context, projectID int64, issueIID int64) ([]issuedomain.ProjectIssueComment, error) {
	issue, err := s.loadIssue(ctx, projectID, issueIID)
	if err != nil {
		return nil, err
	}
	items, err := s.commentRepo.ListByIssueID(ctx, issue.ID)
	if err != nil {
		return nil, err
	}
	return items.Values(), nil
}

func (s *Service) CreateComment(ctx context.Context, projectID int64, issueIID int64, input CreateCommentInput) (issuedomain.ProjectIssueComment, error) {
	issue, err := s.loadIssue(ctx, projectID, issueIID)
	if err != nil {
		return issuedomain.ProjectIssueComment{}, err
	}
	if strings.TrimSpace(input.Body) == "" {
		return issuedomain.ProjectIssueComment{}, oops.In("issue").With("project_id", projectID, "issue_iid", issueIID, "author_user_id", input.AuthorUserID).New("issue comment body is required")
	}
	if _, err := s.userRepo.GetByID(ctx, input.AuthorUserID); err != nil {
		return issuedomain.ProjectIssueComment{}, apperror.NotFound("comment author not found", err)
	}
	return s.commentRepo.Create(ctx, storageports.CreateProjectIssueCommentInput{ProjectIssueID: issue.ID, AuthorUserID: input.AuthorUserID, Body: input.Body})
}

func (s *Service) ListAttachments(ctx context.Context, projectID int64, issueIID int64) ([]issuedomain.ProjectIssueAttachment, error) {
	issue, err := s.loadIssue(ctx, projectID, issueIID)
	if err != nil {
		return nil, err
	}
	items, err := s.attachmentRepo.ListByIssueID(ctx, issue.ID)
	if err != nil {
		return nil, err
	}
	return items.Values(), nil
}

func (s *Service) CreateAttachment(ctx context.Context, projectID int64, issueIID int64, input CreateAttachmentInput) (issuedomain.ProjectIssueAttachment, error) {
	if strings.TrimSpace(input.ContentBase64) == "" {
		return issuedomain.ProjectIssueAttachment{}, oops.In("issue").With("project_id", projectID, "issue_iid", issueIID, "file_name", input.FileName).New("attachment content_base64 is required")
	}
	content, err := base64.StdEncoding.DecodeString(strings.TrimSpace(input.ContentBase64))
	if err != nil {
		return issuedomain.ProjectIssueAttachment{}, oops.In("issue").With("project_id", projectID, "issue_iid", issueIID, "file_name", input.FileName).Wrapf(err, "decode attachment content")
	}
	uploaded, err := s.UploadAttachment(ctx, projectID, AttachmentUploadInput{
		UploadedByUserID: input.UploadedByUserID,
		IssueIID:         issueIID,
		FileName:         input.FileName,
		ContentType:      input.ContentType,
		Content:          content,
	})
	if err != nil {
		return issuedomain.ProjectIssueAttachment{}, err
	}
	if uploaded.Attachment == nil {
		return issuedomain.ProjectIssueAttachment{}, oops.In("issue").With("project_id", projectID, "issue_iid", issueIID, "file_name", input.FileName).New("issue attachment record was not created")
	}
	return *uploaded.Attachment, nil
}

func (s *Service) UploadAttachment(ctx context.Context, projectID int64, input AttachmentUploadInput) (AttachmentUploadView, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return AttachmentUploadView{}, apperror.NotFound("project not found", err)
	}
	if _, lookupErr := s.userRepo.GetByID(ctx, input.UploadedByUserID); lookupErr != nil {
		return AttachmentUploadView{}, apperror.NotFound("attachment uploader not found", lookupErr)
	}
	if strings.TrimSpace(input.FileName) == "" {
		return AttachmentUploadView{}, oops.In("issue").With("project_id", projectID, "issue_iid", input.IssueIID, "uploaded_by_user_id", input.UploadedByUserID).New("attachment file_name is required")
	}
	if len(input.Content) == 0 {
		return AttachmentUploadView{}, oops.In("issue").With("project_id", projectID, "issue_iid", input.IssueIID, "file_name", input.FileName).New("attachment content is required")
	}
	contentType := strings.TrimSpace(input.ContentType)
	if contentType == "" {
		contentType = storageports.DetectContentType(input.FileName)
	}

	if input.IssueIID <= 0 {
		token, tokenErr := randomToken()
		if tokenErr != nil {
			return AttachmentUploadView{}, tokenErr
		}
		storageKey := storageports.BuildIssueDraftStorageKey(project.FullPath, token, input.FileName)
		if saveErr := s.storage.SaveObject(ctx, storageKey, input.Content, contentType); saveErr != nil {
			return AttachmentUploadView{}, saveErr
		}
		return AttachmentUploadView{
			ObjectKey:   storageKey,
			FileName:    strings.TrimSpace(input.FileName),
			ContentType: contentType,
			ByteSize:    int64(len(input.Content)),
		}, nil
	}

	issue, err := s.loadIssue(ctx, projectID, input.IssueIID)
	if err != nil {
		return AttachmentUploadView{}, err
	}
	attachment, err := s.attachmentRepo.Create(ctx, storageports.CreateProjectIssueAttachmentInput{ProjectIssueID: issue.ID, UploadedByUserID: input.UploadedByUserID, FileName: input.FileName, ContentType: contentType})
	if err != nil {
		return AttachmentUploadView{}, err
	}
	storageKey, err := s.storage.SaveIssueAttachment(ctx, project.FullPath, issue.IID, attachment.ID, attachment.FileName, input.Content, contentType)
	if err != nil {
		if cleanupErr := s.attachmentRepo.DeleteByID(ctx, attachment.ID); cleanupErr != nil {
			return AttachmentUploadView{}, oops.In("issue").With("project_id", projectID, "issue_id", issue.ID, "attachment_id", attachment.ID).Wrapf(oops.Join(err, cleanupErr), "save issue attachment and cleanup record")
		}
		return AttachmentUploadView{}, oops.In("issue").With("project_id", projectID, "issue_id", issue.ID, "attachment_id", attachment.ID).Wrapf(err, "save issue attachment")
	}
	if storeErr := s.attachmentRepo.MarkStored(ctx, attachment.ID, storageports.StoreProjectIssueAttachmentInput{ContentType: contentType, ByteSize: int64(len(input.Content)), StorageKey: storageKey}); storeErr != nil {
		return AttachmentUploadView{}, storeErr
	}
	stored, err := s.attachmentRepo.GetByIssueAndID(ctx, issue.ID, attachment.ID)
	if err != nil {
		return AttachmentUploadView{}, err
	}
	return AttachmentUploadView{
		Attachment:  &stored,
		ObjectKey:   stored.StorageKey,
		FileName:    stored.FileName,
		ContentType: stored.ContentType,
		ByteSize:    stored.ByteSize,
	}, nil
}

func (s *Service) GetAttachmentContent(ctx context.Context, projectID int64, issueIID int64, attachmentID int64) (AttachmentContentView, error) {
	raw, attachment, err := s.loadAttachmentRaw(ctx, projectID, issueIID, attachmentID)
	if err != nil {
		return AttachmentContentView{}, err
	}
	return AttachmentContentView{Attachment: attachment, Content: base64.StdEncoding.EncodeToString(raw.Content)}, nil
}

func (s *Service) GetAttachmentRaw(ctx context.Context, projectID int64, issueIID int64, attachmentID int64) (AttachmentRawContent, error) {
	raw, _, err := s.loadAttachmentRaw(ctx, projectID, issueIID, attachmentID)
	return raw, err
}

func (s *Service) GetDraftAttachmentRaw(ctx context.Context, projectID int64, objectKey string) (AttachmentRawContent, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return AttachmentRawContent{}, apperror.NotFound("project not found", err)
	}
	normalizedKey := strings.Trim(strings.ReplaceAll(objectKey, "\\", "/"), "/")
	prefix := storageports.BuildIssueDraftStoragePrefix(project.FullPath) + "/"
	if !strings.HasPrefix(normalizedKey, prefix) {
		return AttachmentRawContent{}, apperror.BadRequest("invalid attachment object key", nil)
	}
	content, err := s.storage.Load(ctx, normalizedKey)
	if err != nil {
		s.logger.Error("load issue draft attachment failed", slog.String("storage_key", normalizedKey), slog.String("error", err.Error()))
		return AttachmentRawContent{}, apperror.NotFound("issue attachment content not found", err)
	}
	return AttachmentRawContent{
		FileName:    storageKeyFileName(normalizedKey),
		ContentType: storageports.DetectContentType(normalizedKey),
		Content:     content,
	}, nil
}

func (s *Service) loadAttachmentRaw(ctx context.Context, projectID int64, issueIID int64, attachmentID int64) (AttachmentRawContent, issuedomain.ProjectIssueAttachment, error) {
	issue, err := s.loadIssue(ctx, projectID, issueIID)
	if err != nil {
		return AttachmentRawContent{}, issuedomain.ProjectIssueAttachment{}, err
	}
	attachment, err := s.attachmentRepo.GetByIssueAndID(ctx, issue.ID, attachmentID)
	if err != nil {
		if errors.Is(err, storageports.ErrNotFound) {
			return AttachmentRawContent{}, issuedomain.ProjectIssueAttachment{}, apperror.NotFound("issue attachment not found", err)
		}
		return AttachmentRawContent{}, issuedomain.ProjectIssueAttachment{}, err
	}
	content, err := s.storage.Load(ctx, attachment.StorageKey)
	if err != nil {
		s.logger.Error("load issue attachment failed", slog.String("storage_key", attachment.StorageKey), slog.String("error", err.Error()))
		return AttachmentRawContent{}, issuedomain.ProjectIssueAttachment{}, apperror.NotFound("issue attachment content not found", err)
	}
	return AttachmentRawContent{FileName: attachment.FileName, ContentType: attachment.ContentType, Content: content}, attachment, nil
}

func (s *Service) loadIssue(ctx context.Context, projectID int64, issueIID int64) (issuedomain.ProjectIssue, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return issuedomain.ProjectIssue{}, apperror.NotFound("project not found", err)
	}
	issue, err := s.issueRepo.GetByProjectAndIID(ctx, projectID, issueIID)
	if err != nil {
		if errors.Is(err, storageports.ErrNotFound) {
			return issuedomain.ProjectIssue{}, apperror.NotFound("issue not found", err)
		}
		return issuedomain.ProjectIssue{}, err
	}
	return issue, nil
}

func randomToken() (string, error) {
	var data [16]byte
	if _, err := rand.Read(data[:]); err != nil {
		return "", oops.In("issue").Wrapf(err, "generate attachment token")
	}
	return hex.EncodeToString(data[:]), nil
}

func storageKeyFileName(key string) string {
	key = strings.Trim(strings.ReplaceAll(key, "\\", "/"), "/")
	if key == "" {
		return "blob.bin"
	}
	parts := strings.Split(key, "/")
	return parts[len(parts)-1]
}
