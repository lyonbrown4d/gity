package projectissueattachment

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/DaiYuANg/gity/internal/entity"

	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
)

type Repository struct {
	base *dbxrepo.Base[entity.ProjectIssueAttachment, entity.ProjectIssueAttachmentSchemaDef]
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
		base: dbxrepo.NewWithOptions[entity.ProjectIssueAttachment](db, entity.ProjectIssueAttachmentSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByIssueID(ctx context.Context, issueID int64) (*collectionx.List[entity.ProjectIssueAttachment], error) {
	query := querydsl.Select(entity.ProjectIssueAttachmentSchema.AllColumns().Values()...).
		From(entity.ProjectIssueAttachmentSchema).
		Where(entity.ProjectIssueAttachmentSchema.ProjectIssueID.Eq(issueID)).
		OrderBy(entity.ProjectIssueAttachmentSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByIssueAndID(ctx context.Context, issueID int64, attachmentID int64) (entity.ProjectIssueAttachment, error) {
	query := querydsl.Select(entity.ProjectIssueAttachmentSchema.AllColumns().Values()...).
		From(entity.ProjectIssueAttachmentSchema).
		Where(querydsl.And(
			entity.ProjectIssueAttachmentSchema.ProjectIssueID.Eq(issueID),
			entity.ProjectIssueAttachmentSchema.ID.Eq(attachmentID),
		)).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.ProjectIssueAttachment, error) {
	now := time.Now().UTC()
	item := entity.ProjectIssueAttachment{
		ProjectIssueID:   input.ProjectIssueID,
		UploadedByUserID: input.UploadedByUserID,
		FileName:         strings.TrimSpace(input.FileName),
		ContentType:      strings.TrimSpace(input.ContentType),
		StorageKey:       fmt.Sprintf("pending/%d", time.Now().UnixNano()),
		CreatedAt:        now,
		UpdatedAt:        now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return entity.ProjectIssueAttachment{}, fmt.Errorf("insert project issue attachment: %w", err)
	}
	return item, nil
}

func (r *Repository) MarkStored(ctx context.Context, attachmentID int64, input StoreInput) error {
	_, err := r.base.UpdateByID(
		ctx,
		attachmentID,
		entity.ProjectIssueAttachmentSchema.ContentType.Set(strings.TrimSpace(input.ContentType)),
		entity.ProjectIssueAttachmentSchema.ByteSize.Set(input.ByteSize),
		entity.ProjectIssueAttachmentSchema.StorageKey.Set(strings.TrimSpace(input.StorageKey)),
		entity.ProjectIssueAttachmentSchema.UpdatedAt.Set(time.Now().UTC()),
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
