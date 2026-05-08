package issue

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"encoding/hex"
	"errors"
	"log/slog"
	"strings"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	storageports "github.com/DaiYuANg/gity/internal/application/ports"
	issuedomain "github.com/DaiYuANg/gity/internal/domain/issue"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	"github.com/samber/oops"
)

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

func (s *Service) ListAttachments(ctx context.Context, projectID, issueIID int64) ([]issuedomain.ProjectIssueAttachment, error) {
	issue, err := s.loadIssue(ctx, projectID, issueIID)
	if err != nil {
		return nil, err
	}
	items, err := s.attachmentRepo.ListByIssueID(ctx, issue.ID)
	if err != nil {
		return nil, oops.In("issue").With("project_id", projectID, "issue_id", issue.ID, "issue_iid", issueIID).Wrapf(err, "list issue attachments")
	}
	return items.Values(), nil
}

func (s *Service) CreateAttachment(ctx context.Context, projectID, issueIID int64, input CreateAttachmentInput) (issuedomain.ProjectIssueAttachment, error) {
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
	project, contentType, err := s.validateAttachmentUpload(ctx, projectID, input)
	if err != nil {
		return AttachmentUploadView{}, err
	}
	if input.IssueIID <= 0 {
		return s.uploadDraftAttachment(ctx, project, input, contentType)
	}
	return s.uploadIssueAttachment(ctx, project, input, contentType)
}

func (s *Service) validateAttachmentUpload(ctx context.Context, projectID int64, input AttachmentUploadInput) (projectdomain.Project, string, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return projectdomain.Project{}, "", apperror.NotFound("project not found", err)
	}
	if _, lookupErr := s.userRepo.GetByID(ctx, input.UploadedByUserID); lookupErr != nil {
		return projectdomain.Project{}, "", apperror.NotFound("attachment uploader not found", lookupErr)
	}
	if strings.TrimSpace(input.FileName) == "" {
		return projectdomain.Project{}, "", oops.In("issue").With("project_id", projectID, "issue_iid", input.IssueIID, "uploaded_by_user_id", input.UploadedByUserID).New("attachment file_name is required")
	}
	if len(input.Content) == 0 {
		return projectdomain.Project{}, "", oops.In("issue").With("project_id", projectID, "issue_iid", input.IssueIID, "file_name", input.FileName).New("attachment content is required")
	}
	contentType := strings.TrimSpace(input.ContentType)
	if contentType == "" {
		contentType = storageports.DetectContentType(input.FileName)
	}
	return project, contentType, nil
}

func (s *Service) uploadDraftAttachment(ctx context.Context, project projectdomain.Project, input AttachmentUploadInput, contentType string) (AttachmentUploadView, error) {
	token, tokenErr := randomToken()
	if tokenErr != nil {
		return AttachmentUploadView{}, tokenErr
	}
	storageKey := storageports.BuildIssueDraftStorageKey(project.FullPath, token, input.FileName)
	if saveErr := s.storage.SaveObject(ctx, storageKey, input.Content, contentType); saveErr != nil {
		return AttachmentUploadView{}, oops.In("issue").With("project_id", project.ID, "issue_iid", input.IssueIID, "storage_key", storageKey).Wrapf(saveErr, "save issue draft attachment")
	}
	return AttachmentUploadView{
		ObjectKey:   storageKey,
		FileName:    strings.TrimSpace(input.FileName),
		ContentType: contentType,
		ByteSize:    int64(len(input.Content)),
	}, nil
}

func (s *Service) uploadIssueAttachment(ctx context.Context, project projectdomain.Project, input AttachmentUploadInput, contentType string) (AttachmentUploadView, error) {
	issue, err := s.loadIssue(ctx, project.ID, input.IssueIID)
	if err != nil {
		return AttachmentUploadView{}, err
	}
	attachment, err := s.attachmentRepo.Create(ctx, storageports.CreateProjectIssueAttachmentInput{ProjectIssueID: issue.ID, UploadedByUserID: input.UploadedByUserID, FileName: input.FileName, ContentType: contentType})
	if err != nil {
		return AttachmentUploadView{}, oops.In("issue").With("project_id", project.ID, "issue_id", issue.ID, "issue_iid", input.IssueIID, "uploaded_by_user_id", input.UploadedByUserID).Wrapf(err, "create issue attachment record")
	}
	storageKey, err := s.storage.SaveIssueAttachment(ctx, project.FullPath, issue.IID, attachment.ID, attachment.FileName, input.Content, contentType)
	if err != nil {
		if cleanupErr := s.attachmentRepo.DeleteByID(ctx, attachment.ID); cleanupErr != nil {
			return AttachmentUploadView{}, oops.In("issue").With("project_id", project.ID, "issue_id", issue.ID, "attachment_id", attachment.ID).Wrapf(oops.Join(err, cleanupErr), "save issue attachment and cleanup record")
		}
		return AttachmentUploadView{}, oops.In("issue").With("project_id", project.ID, "issue_id", issue.ID, "attachment_id", attachment.ID).Wrapf(err, "save issue attachment")
	}
	if storeErr := s.attachmentRepo.MarkStored(ctx, attachment.ID, storageports.StoreProjectIssueAttachmentInput{ContentType: contentType, ByteSize: int64(len(input.Content)), StorageKey: storageKey}); storeErr != nil {
		return AttachmentUploadView{}, oops.In("issue").With("project_id", project.ID, "issue_id", issue.ID, "attachment_id", attachment.ID, "storage_key", storageKey).Wrapf(storeErr, "mark issue attachment stored")
	}
	stored, err := s.attachmentRepo.GetByIssueAndID(ctx, issue.ID, attachment.ID)
	if err != nil {
		return AttachmentUploadView{}, oops.In("issue").With("project_id", project.ID, "issue_id", issue.ID, "attachment_id", attachment.ID).Wrapf(err, "load stored issue attachment")
	}
	return AttachmentUploadView{
		Attachment:  &stored,
		ObjectKey:   stored.StorageKey,
		FileName:    stored.FileName,
		ContentType: stored.ContentType,
		ByteSize:    stored.ByteSize,
	}, nil
}

func (s *Service) GetAttachmentContent(ctx context.Context, projectID, issueIID, attachmentID int64) (AttachmentContentView, error) {
	raw, attachment, err := s.loadAttachmentRaw(ctx, projectID, issueIID, attachmentID)
	if err != nil {
		return AttachmentContentView{}, err
	}
	return AttachmentContentView{Attachment: attachment, Content: base64.StdEncoding.EncodeToString(raw.Content)}, nil
}

func (s *Service) GetAttachmentRaw(ctx context.Context, projectID, issueIID, attachmentID int64) (AttachmentRawContent, error) {
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

func (s *Service) loadAttachmentRaw(ctx context.Context, projectID, issueIID, attachmentID int64) (AttachmentRawContent, issuedomain.ProjectIssueAttachment, error) {
	issue, err := s.loadIssue(ctx, projectID, issueIID)
	if err != nil {
		return AttachmentRawContent{}, issuedomain.ProjectIssueAttachment{}, err
	}
	attachment, err := s.attachmentRepo.GetByIssueAndID(ctx, issue.ID, attachmentID)
	if err != nil {
		if errors.Is(err, storageports.ErrNotFound) {
			return AttachmentRawContent{}, issuedomain.ProjectIssueAttachment{}, apperror.NotFound("issue attachment not found", err)
		}
		return AttachmentRawContent{}, issuedomain.ProjectIssueAttachment{}, oops.In("issue").With("project_id", projectID, "issue_id", issue.ID, "issue_iid", issueIID, "attachment_id", attachmentID).Wrapf(err, "load issue attachment")
	}
	content, err := s.storage.Load(ctx, attachment.StorageKey)
	if err != nil {
		s.logger.Error("load issue attachment failed", slog.String("storage_key", attachment.StorageKey), slog.String("error", err.Error()))
		return AttachmentRawContent{}, issuedomain.ProjectIssueAttachment{}, apperror.NotFound("issue attachment content not found", err)
	}
	return AttachmentRawContent{FileName: attachment.FileName, ContentType: attachment.ContentType, Content: content}, attachment, nil
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
