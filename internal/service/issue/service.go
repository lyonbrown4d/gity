package issue

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"encoding/hex"
	"fmt"
	"log/slog"
	"net/http"
	"strings"

	"github.com/DaiYuANg/gity/internal/entity"
	platformstorage "github.com/DaiYuANg/gity/internal/platform/storage"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	projectissuerepo "github.com/DaiYuANg/gity/internal/repository/projectissue"
	projectissueattachmentrepo "github.com/DaiYuANg/gity/internal/repository/projectissueattachment"
	projectissuecommentrepo "github.com/DaiYuANg/gity/internal/repository/projectissuecomment"
	userrepo "github.com/DaiYuANg/gity/internal/repository/user"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"github.com/arcgolabs/httpx"
)

type Service struct {
	logger         *slog.Logger
	projectRepo    *projectrepo.Repository
	issueRepo      *projectissuerepo.Repository
	commentRepo    *projectissuecommentrepo.Repository
	attachmentRepo *projectissueattachmentrepo.Repository
	userRepo       *userrepo.Repository
	storage        *platformstorage.Service
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
	Attachment entity.ProjectIssueAttachment `json:"attachment"`
	Content    string                        `json:"content_base64"`
}

type AttachmentUploadInput struct {
	UploadedByUserID int64
	IssueIID         int64
	FileName         string
	ContentType      string
	Content          []byte
}

type AttachmentUploadView struct {
	Attachment  *entity.ProjectIssueAttachment
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

func NewService(projectRepo *projectrepo.Repository, issueRepo *projectissuerepo.Repository, commentRepo *projectissuecommentrepo.Repository, attachmentRepo *projectissueattachmentrepo.Repository, userRepo *userrepo.Repository, storage *platformstorage.Service) *Service {
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

func (s *Service) ListIssues(ctx context.Context, projectID int64) ([]entity.ProjectIssue, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	items, err := s.issueRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, err
	}
	return items.Values(), nil
}

func (s *Service) GetIssueByIID(ctx context.Context, projectID int64, issueIID int64) (entity.ProjectIssue, error) {
	return s.loadIssue(ctx, projectID, issueIID)
}

func (s *Service) CreateIssue(ctx context.Context, projectID int64, input CreateIssueInput) (entity.ProjectIssue, error) {
	if strings.TrimSpace(input.Title) == "" {
		return entity.ProjectIssue{}, fmt.Errorf("issue title is required")
	}
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return entity.ProjectIssue{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	if _, err := s.userRepo.GetByID(ctx, input.AuthorUserID); err != nil {
		return entity.ProjectIssue{}, httpx.NewError(http.StatusNotFound, "issue author not found", err)
	}
	return s.issueRepo.Create(ctx, projectissuerepo.CreateInput{
		ProjectID:    projectID,
		AuthorUserID: input.AuthorUserID,
		Title:        input.Title,
		Description:  input.Description,
	})
}

func (s *Service) UpdateIssue(ctx context.Context, projectID int64, issueIID int64, input UpdateIssueInput) (entity.ProjectIssue, error) {
	issue, err := s.loadIssue(ctx, projectID, issueIID)
	if err != nil {
		return entity.ProjectIssue{}, err
	}
	if input.Title != nil && strings.TrimSpace(*input.Title) == "" {
		return entity.ProjectIssue{}, fmt.Errorf("issue title is required")
	}
	if input.State != nil {
		state := strings.TrimSpace(*input.State)
		if state != "opened" && state != "closed" {
			return entity.ProjectIssue{}, fmt.Errorf("issue state must be opened or closed")
		}
	}
	if err := s.issueRepo.UpdateByID(ctx, issue.ID, projectissuerepo.UpdateInput{Title: input.Title, Description: input.Description, State: input.State}); err != nil {
		return entity.ProjectIssue{}, err
	}
	return s.loadIssue(ctx, projectID, issueIID)
}

func (s *Service) ListComments(ctx context.Context, projectID int64, issueIID int64) ([]entity.ProjectIssueComment, error) {
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

func (s *Service) CreateComment(ctx context.Context, projectID int64, issueIID int64, input CreateCommentInput) (entity.ProjectIssueComment, error) {
	issue, err := s.loadIssue(ctx, projectID, issueIID)
	if err != nil {
		return entity.ProjectIssueComment{}, err
	}
	if strings.TrimSpace(input.Body) == "" {
		return entity.ProjectIssueComment{}, fmt.Errorf("issue comment body is required")
	}
	if _, err := s.userRepo.GetByID(ctx, input.AuthorUserID); err != nil {
		return entity.ProjectIssueComment{}, httpx.NewError(http.StatusNotFound, "comment author not found", err)
	}
	return s.commentRepo.Create(ctx, projectissuecommentrepo.CreateInput{ProjectIssueID: issue.ID, AuthorUserID: input.AuthorUserID, Body: input.Body})
}

func (s *Service) ListAttachments(ctx context.Context, projectID int64, issueIID int64) ([]entity.ProjectIssueAttachment, error) {
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

func (s *Service) CreateAttachment(ctx context.Context, projectID int64, issueIID int64, input CreateAttachmentInput) (entity.ProjectIssueAttachment, error) {
	if strings.TrimSpace(input.ContentBase64) == "" {
		return entity.ProjectIssueAttachment{}, fmt.Errorf("attachment content_base64 is required")
	}
	content, err := base64.StdEncoding.DecodeString(strings.TrimSpace(input.ContentBase64))
	if err != nil {
		return entity.ProjectIssueAttachment{}, fmt.Errorf("decode attachment content: %w", err)
	}
	uploaded, err := s.UploadAttachment(ctx, projectID, AttachmentUploadInput{
		UploadedByUserID: input.UploadedByUserID,
		IssueIID:         issueIID,
		FileName:         input.FileName,
		ContentType:      input.ContentType,
		Content:          content,
	})
	if err != nil {
		return entity.ProjectIssueAttachment{}, err
	}
	if uploaded.Attachment == nil {
		return entity.ProjectIssueAttachment{}, fmt.Errorf("issue attachment record was not created")
	}
	return *uploaded.Attachment, nil
}

func (s *Service) UploadAttachment(ctx context.Context, projectID int64, input AttachmentUploadInput) (AttachmentUploadView, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return AttachmentUploadView{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	if _, err := s.userRepo.GetByID(ctx, input.UploadedByUserID); err != nil {
		return AttachmentUploadView{}, httpx.NewError(http.StatusNotFound, "attachment uploader not found", err)
	}
	if strings.TrimSpace(input.FileName) == "" {
		return AttachmentUploadView{}, fmt.Errorf("attachment file_name is required")
	}
	if len(input.Content) == 0 {
		return AttachmentUploadView{}, fmt.Errorf("attachment content is required")
	}
	contentType := strings.TrimSpace(input.ContentType)
	if contentType == "" {
		contentType = platformstorage.DetectContentType(input.FileName)
	}

	if input.IssueIID <= 0 {
		token, err := randomToken()
		if err != nil {
			return AttachmentUploadView{}, err
		}
		storageKey := platformstorage.BuildIssueDraftStorageKey(project.FullPath, token, input.FileName)
		if err := s.storage.SaveObject(ctx, storageKey, input.Content, contentType); err != nil {
			return AttachmentUploadView{}, err
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
	attachment, err := s.attachmentRepo.Create(ctx, projectissueattachmentrepo.CreateInput{ProjectIssueID: issue.ID, UploadedByUserID: input.UploadedByUserID, FileName: input.FileName, ContentType: contentType})
	if err != nil {
		return AttachmentUploadView{}, err
	}
	storageKey, err := s.storage.SaveIssueAttachment(ctx, project.FullPath, issue.IID, attachment.ID, attachment.FileName, input.Content, contentType)
	if err != nil {
		_ = s.attachmentRepo.DeleteByID(ctx, attachment.ID)
		return AttachmentUploadView{}, err
	}
	if err := s.attachmentRepo.MarkStored(ctx, attachment.ID, projectissueattachmentrepo.StoreInput{ContentType: contentType, ByteSize: int64(len(input.Content)), StorageKey: storageKey}); err != nil {
		return AttachmentUploadView{}, err
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
		return AttachmentRawContent{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	normalizedKey := strings.Trim(strings.ReplaceAll(objectKey, "\\", "/"), "/")
	prefix := platformstorage.BuildIssueDraftStoragePrefix(project.FullPath) + "/"
	if !strings.HasPrefix(normalizedKey, prefix) {
		return AttachmentRawContent{}, httpx.NewError(http.StatusBadRequest, "invalid attachment object key", nil)
	}
	content, err := s.storage.Load(ctx, normalizedKey)
	if err != nil {
		s.logger.Error("load issue draft attachment failed", slog.String("storage_key", normalizedKey), slog.String("error", err.Error()))
		return AttachmentRawContent{}, httpx.NewError(http.StatusNotFound, "issue attachment content not found", err)
	}
	return AttachmentRawContent{
		FileName:    storageKeyFileName(normalizedKey),
		ContentType: platformstorage.DetectContentType(normalizedKey),
		Content:     content,
	}, nil
}

func (s *Service) loadAttachmentRaw(ctx context.Context, projectID int64, issueIID int64, attachmentID int64) (AttachmentRawContent, entity.ProjectIssueAttachment, error) {
	issue, err := s.loadIssue(ctx, projectID, issueIID)
	if err != nil {
		return AttachmentRawContent{}, entity.ProjectIssueAttachment{}, err
	}
	attachment, err := s.attachmentRepo.GetByIssueAndID(ctx, issue.ID, attachmentID)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return AttachmentRawContent{}, entity.ProjectIssueAttachment{}, httpx.NewError(http.StatusNotFound, "issue attachment not found", err)
		}
		return AttachmentRawContent{}, entity.ProjectIssueAttachment{}, err
	}
	content, err := s.storage.Load(ctx, attachment.StorageKey)
	if err != nil {
		s.logger.Error("load issue attachment failed", slog.String("storage_key", attachment.StorageKey), slog.String("error", err.Error()))
		return AttachmentRawContent{}, entity.ProjectIssueAttachment{}, httpx.NewError(http.StatusNotFound, "issue attachment content not found", err)
	}
	return AttachmentRawContent{FileName: attachment.FileName, ContentType: attachment.ContentType, Content: content}, attachment, nil
}

func (s *Service) loadIssue(ctx context.Context, projectID int64, issueIID int64) (entity.ProjectIssue, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return entity.ProjectIssue{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	issue, err := s.issueRepo.GetByProjectAndIID(ctx, projectID, issueIID)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return entity.ProjectIssue{}, httpx.NewError(http.StatusNotFound, "issue not found", err)
		}
		return entity.ProjectIssue{}, err
	}
	return issue, nil
}

func randomToken() (string, error) {
	var data [16]byte
	if _, err := rand.Read(data[:]); err != nil {
		return "", fmt.Errorf("generate attachment token: %w", err)
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
