package projectmergerequest

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/DaiYuANg/arcgo/collectionx"
	"github.com/DaiYuANg/arcgo/dbx"
	dbxrepo "github.com/DaiYuANg/arcgo/dbx/repository"
	"github.com/DaiYuANg/gity/internal/entity"
)

type Repository struct {
	base *dbxrepo.Base[entity.ProjectMergeRequest, entity.ProjectMergeRequestSchemaDef]
}

type CreateInput struct {
	ProjectID    int64
	AuthorUserID int64
	Title        string
	Description  string
	SourceBranch string
	TargetBranch string
}

type UpdateInput struct {
	Title       *string
	Description *string
	State       *string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[entity.ProjectMergeRequest](db, entity.ProjectMergeRequestSchema, dbxrepo.WithByIDNotFoundAsError(true))}, nil
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (collectionx.List[entity.ProjectMergeRequest], error) {
	query := dbx.Select(entity.ProjectMergeRequestSchema.AllColumns().Values()...).From(entity.ProjectMergeRequestSchema).Where(entity.ProjectMergeRequestSchema.ProjectID.Eq(projectID)).OrderBy(entity.ProjectMergeRequestSchema.IID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByProjectAndIID(ctx context.Context, projectID int64, iid int64) (entity.ProjectMergeRequest, error) {
	query := dbx.Select(entity.ProjectMergeRequestSchema.AllColumns().Values()...).From(entity.ProjectMergeRequestSchema).Where(entity.ProjectMergeRequestSchema.ProjectID.Eq(projectID)).Where(entity.ProjectMergeRequestSchema.IID.Eq(iid)).Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.ProjectMergeRequest, error) {
	var created entity.ProjectMergeRequest
	err := r.base.InTx(ctx, nil, func(_ *dbx.Tx, repo *dbxrepo.Base[entity.ProjectMergeRequest, entity.ProjectMergeRequestSchemaDef]) error {
		nextIID := int64(1)
		query := dbx.Select(entity.ProjectMergeRequestSchema.AllColumns().Values()...).From(entity.ProjectMergeRequestSchema).Where(entity.ProjectMergeRequestSchema.ProjectID.Eq(input.ProjectID)).OrderBy(entity.ProjectMergeRequestSchema.IID.Desc()).Limit(1)
		last, err := repo.First(ctx, query)
		if err == nil {
			nextIID = last.IID + 1
		} else if err != nil && err != dbxrepo.ErrNotFound {
			return err
		}
		now := time.Now().UTC()
		item := entity.ProjectMergeRequest{
			ProjectID:    input.ProjectID,
			IID:          nextIID,
			AuthorUserID: input.AuthorUserID,
			Title:        strings.TrimSpace(input.Title),
			Description:  strings.TrimSpace(input.Description),
			State:        "opened",
			SourceBranch: strings.TrimSpace(input.SourceBranch),
			TargetBranch: strings.TrimSpace(input.TargetBranch),
			CreatedAt:    now,
			UpdatedAt:    now,
		}
		if err := repo.Create(ctx, &item); err != nil {
			return fmt.Errorf("insert merge request: %w", err)
		}
		created = item
		return nil
	})
	if err != nil {
		return entity.ProjectMergeRequest{}, err
	}
	return created, nil
}

func (r *Repository) UpdateByID(ctx context.Context, id int64, input UpdateInput) error {
	assignments := make([]dbx.Assignment, 0, 4)
	if input.Title != nil {
		assignments = append(assignments, entity.ProjectMergeRequestSchema.Title.Set(strings.TrimSpace(*input.Title)))
	}
	if input.Description != nil {
		assignments = append(assignments, entity.ProjectMergeRequestSchema.Description.Set(strings.TrimSpace(*input.Description)))
	}
	if input.State != nil {
		assignments = append(assignments, entity.ProjectMergeRequestSchema.State.Set(strings.TrimSpace(*input.State)))
	}
	assignments = append(assignments, entity.ProjectMergeRequestSchema.UpdatedAt.Set(time.Now().UTC()))
	if _, err := r.base.UpdateByID(ctx, id, assignments...); err != nil {
		return fmt.Errorf("update merge request: %w", err)
	}
	return nil
}
