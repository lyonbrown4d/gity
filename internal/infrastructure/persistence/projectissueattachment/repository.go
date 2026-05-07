package projectissueattachment

import (
	"context"
	"fmt"
	issuedomain "github.com/DaiYuANg/gity/internal/domain/issue"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[issuedomain.ProjectIssueAttachment, issuedomain.ProjectIssueAttachmentSchemaDef]
}

type CreateInput struct {
	ProjectIssueID   int64
	UploadedByUserID int64
	FileName         string
	ContentType      string
}

type StoreInput struct {
	ContentType string
	ByteSize    int64
	StorageKey  string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[issuedomain.ProjectIssueAttachment](db, issuedomain.ProjectIssueAttachmentSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByIssueID(ctx context.Context, issueID int64) (*collectionx.List[issuedomain.ProjectIssueAttachment], error) {
	query := querydsl.Select(issuedomain.ProjectIssueAttachmentSchema.AllColumns().Values()...).
		From(issuedomain.ProjectIssueAttachmentSchema).
		Where(issuedomain.ProjectIssueAttachmentSchema.ProjectIssueID.Eq(issueID)).
		OrderBy(issuedomain.ProjectIssueAttachmentSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByIssueAndID(ctx context.Context, issueID int64, attachmentID int64) (issuedomain.ProjectIssueAttachment, error) {
	query := querydsl.Select(issuedomain.ProjectIssueAttachmentSchema.AllColumns().Values()...).
		From(issuedomain.ProjectIssueAttachmentSchema).
		Where(querydsl.And(
			issuedomain.ProjectIssueAttachmentSchema.ProjectIssueID.Eq(issueID),
			issuedomain.ProjectIssueAttachmentSchema.ID.Eq(attachmentID),
		)).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (issuedomain.ProjectIssueAttachment, error) {
	now := time.Now().UTC()
	item := issuedomain.ProjectIssueAttachment{
		ProjectIssueID:   input.ProjectIssueID,
		UploadedByUserID: input.UploadedByUserID,
		FileName:         strings.TrimSpace(input.FileName),
		ContentType:      strings.TrimSpace(input.ContentType),
		StorageKey:       fmt.Sprintf("pending/%d", time.Now().UnixNano()),
		CreatedAt:        now,
		UpdatedAt:        now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return issuedomain.ProjectIssueAttachment{}, fmt.Errorf("insert project issue attachment: %w", err)
	}
	return item, nil
}

func (r *Repository) MarkStored(ctx context.Context, attachmentID int64, input StoreInput) error {
	_, err := r.base.UpdateByID(
		ctx,
		attachmentID,
		issuedomain.ProjectIssueAttachmentSchema.ContentType.Set(strings.TrimSpace(input.ContentType)),
		issuedomain.ProjectIssueAttachmentSchema.ByteSize.Set(input.ByteSize),
		issuedomain.ProjectIssueAttachmentSchema.StorageKey.Set(strings.TrimSpace(input.StorageKey)),
		issuedomain.ProjectIssueAttachmentSchema.UpdatedAt.Set(time.Now().UTC()),
	)
	if err != nil {
		return fmt.Errorf("update project issue attachment: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByID(ctx context.Context, attachmentID int64) error {
	if _, err := r.base.DeleteByID(ctx, attachmentID); err != nil {
		return fmt.Errorf("delete project issue attachment: %w", err)
	}
	return nil
}
