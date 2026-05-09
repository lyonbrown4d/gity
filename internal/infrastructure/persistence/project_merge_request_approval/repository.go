package projectmergerequestapproval

import (
	"context"
	"errors"
	"time"

	mergeports "github.com/DaiYuANg/gity/internal/application/ports"
	mergedomain "github.com/DaiYuANg/gity/internal/domain/merge"
	persistence "github.com/DaiYuANg/gity/internal/infrastructure/persistence"
	dbschema "github.com/DaiYuANg/gity/internal/infrastructure/persistence/db_schema"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"github.com/samber/oops"
)

type Repository struct {
	base *dbxrepo.Base[mergedomain.ProjectMergeRequestApproval, dbschema.ProjectMergeRequestApprovalSchemaDef]
}

type UpsertInput = mergeports.UpsertProjectMergeRequestApprovalInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[mergedomain.ProjectMergeRequestApproval](db, dbschema.ProjectMergeRequestApprovalSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectMergeRequestApprovalRepository(repo *Repository) mergeports.ProjectMergeRequestApprovalRepository {
	return repo
}

func (r *Repository) ListByMergeRequestID(ctx context.Context, mergeRequestID int64) (*collectionx.List[mergedomain.ProjectMergeRequestApproval], error) {
	query := querydsl.Select(dbschema.ProjectMergeRequestApprovalSchema.AllColumns().Values()...).
		From(dbschema.ProjectMergeRequestApprovalSchema).
		Where(dbschema.ProjectMergeRequestApprovalSchema.MergeRequestID.Eq(mergeRequestID)).
		OrderBy(dbschema.ProjectMergeRequestApprovalSchema.ID.Asc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) Upsert(ctx context.Context, input UpsertInput) (mergedomain.ProjectMergeRequestApproval, error) {
	existing, err := r.getByMergeRequestAndUser(ctx, input.MergeRequestID, input.UserID)
	if err == nil {
		return existing, nil
	}
	if !errors.Is(err, mergeports.ErrNotFound) {
		return mergedomain.ProjectMergeRequestApproval{}, err
	}
	now := time.Now().UTC()
	item := mergedomain.ProjectMergeRequestApproval{
		MergeRequestID: input.MergeRequestID,
		UserID:         input.UserID,
		CreatedAt:      now,
		UpdatedAt:      now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return mergedomain.ProjectMergeRequestApproval{}, oops.In("persistence.merge_request_approval").
			With("merge_request_id", input.MergeRequestID, "user_id", input.UserID).
			Wrapf(err, "insert merge request approval")
	}
	return item, nil
}

func (r *Repository) DeleteByMergeRequestAndUser(ctx context.Context, mergeRequestID, userID int64) error {
	query := querydsl.DeleteFrom(dbschema.ProjectMergeRequestApprovalSchema).
		Where(querydsl.And(
			dbschema.ProjectMergeRequestApprovalSchema.MergeRequestID.Eq(mergeRequestID),
			dbschema.ProjectMergeRequestApprovalSchema.UserID.Eq(userID),
		))
	if _, err := r.base.Delete(ctx, query); err != nil {
		return oops.In("persistence.merge_request_approval").
			With("merge_request_id", mergeRequestID, "user_id", userID).
			Wrapf(err, "delete merge request approval")
	}
	return nil
}

func (r *Repository) getByMergeRequestAndUser(ctx context.Context, mergeRequestID, userID int64) (mergedomain.ProjectMergeRequestApproval, error) {
	query := querydsl.Select(dbschema.ProjectMergeRequestApprovalSchema.AllColumns().Values()...).
		From(dbschema.ProjectMergeRequestApprovalSchema).
		Where(querydsl.And(
			dbschema.ProjectMergeRequestApprovalSchema.MergeRequestID.Eq(mergeRequestID),
			dbschema.ProjectMergeRequestApprovalSchema.UserID.Eq(userID),
		)).
		Limit(1)
	item, err := r.base.First(ctx, query)
	if err != nil {
		if persistence.IsNotFound(err) {
			return mergedomain.ProjectMergeRequestApproval{}, mergeports.ErrNotFound
		}
		return mergedomain.ProjectMergeRequestApproval{}, oops.In("persistence.merge_request_approval").
			With("merge_request_id", mergeRequestID, "user_id", userID).
			Wrapf(err, "load merge request approval")
	}
	return item, nil
}
