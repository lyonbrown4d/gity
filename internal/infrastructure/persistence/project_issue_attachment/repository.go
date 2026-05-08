package projectissueattachment

import (
	"context"
	"fmt"
	issueports "github.com/DaiYuANg/gity/internal/application/ports"
	issuedomain "github.com/DaiYuANg/gity/internal/domain/issue"
	persistence "github.com/DaiYuANg/gity/internal/infrastructure/persistence"
	dbschema "github.com/DaiYuANg/gity/internal/infrastructure/persistence/db_schema"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[issuedomain.ProjectIssueAttachment, dbschema.ProjectIssueAttachmentSchemaDef]
}

type CreateInput = issueports.CreateProjectIssueAttachmentInput

type StoreInput = issueports.StoreProjectIssueAttachmentInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[issuedomain.ProjectIssueAttachment](db, dbschema.ProjectIssueAttachmentSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectIssueAttachmentRepository(repo *Repository) issueports.ProjectIssueAttachmentRepository {
	return repo
}

func (r *Repository) ListByIssueID(ctx context.Context, issueID int64) (*collectionx.List[issuedomain.ProjectIssueAttachment], error) {
	query := querydsl.Select(dbschema.ProjectIssueAttachmentSchema.AllColumns().Values()...).
		From(dbschema.ProjectIssueAttachmentSchema).
		Where(dbschema.ProjectIssueAttachmentSchema.ProjectIssueID.Eq(issueID)).
		OrderBy(dbschema.ProjectIssueAttachmentSchema.ID.Desc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByIssueAndID(ctx context.Context, issueID int64, attachmentID int64) (issuedomain.ProjectIssueAttachment, error) {
	query := querydsl.Select(dbschema.ProjectIssueAttachmentSchema.AllColumns().Values()...).
		From(dbschema.ProjectIssueAttachmentSchema).
		Where(querydsl.And(
			dbschema.ProjectIssueAttachmentSchema.ProjectIssueID.Eq(issueID),
			dbschema.ProjectIssueAttachmentSchema.ID.Eq(attachmentID),
		)).
		Limit(1)
	return persistence.One(r.base.First(ctx, query))
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
	_, err := dbxrepo.By(r.base, dbschema.ProjectIssueAttachmentSchema.ID).Update(
		ctx,
		attachmentID,
		dbschema.ProjectIssueAttachmentSchema.ContentType.Set(strings.TrimSpace(input.ContentType)),
		dbschema.ProjectIssueAttachmentSchema.ByteSize.Set(input.ByteSize),
		dbschema.ProjectIssueAttachmentSchema.StorageKey.Set(strings.TrimSpace(input.StorageKey)),
		dbschema.ProjectIssueAttachmentSchema.UpdatedAt.Set(time.Now().UTC()),
	)
	if err != nil {
		return fmt.Errorf("update project issue attachment: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByID(ctx context.Context, attachmentID int64) error {
	if _, err := dbxrepo.By(r.base, dbschema.ProjectIssueAttachmentSchema.ID).Delete(ctx, attachmentID); err != nil {
		return fmt.Errorf("delete project issue attachment: %w", err)
	}
	return nil
}
