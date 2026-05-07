package projectmergerequest

import (
	"context"
	"fmt"
	mergedomain "github.com/DaiYuANg/gity/internal/domain/merge"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[mergedomain.ProjectMergeRequest, mergedomain.ProjectMergeRequestSchemaDef]
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
	return &Repository{base: dbxrepo.NewWithOptions[mergedomain.ProjectMergeRequest](db, mergedomain.ProjectMergeRequestSchema, dbxrepo.WithByIDNotFoundAsError(true))}, nil
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[mergedomain.ProjectMergeRequest], error) {
	query := querydsl.Select(mergedomain.ProjectMergeRequestSchema.AllColumns().Values()...).From(mergedomain.ProjectMergeRequestSchema).Where(mergedomain.ProjectMergeRequestSchema.ProjectID.Eq(projectID)).OrderBy(mergedomain.ProjectMergeRequestSchema.IID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByProjectAndIID(ctx context.Context, projectID int64, iid int64) (mergedomain.ProjectMergeRequest, error) {
	query := querydsl.Select(mergedomain.ProjectMergeRequestSchema.AllColumns().Values()...).From(mergedomain.ProjectMergeRequestSchema).Where(querydsl.And(mergedomain.ProjectMergeRequestSchema.ProjectID.Eq(projectID), mergedomain.ProjectMergeRequestSchema.IID.Eq(iid))).Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (mergedomain.ProjectMergeRequest, error) {
	var created mergedomain.ProjectMergeRequest
	err := r.base.InTx(ctx, nil, func(_ *dbx.Tx, repo *dbxrepo.Base[mergedomain.ProjectMergeRequest, mergedomain.ProjectMergeRequestSchemaDef]) error {
		nextIID := int64(1)
		query := querydsl.Select(mergedomain.ProjectMergeRequestSchema.AllColumns().Values()...).From(mergedomain.ProjectMergeRequestSchema).Where(mergedomain.ProjectMergeRequestSchema.ProjectID.Eq(input.ProjectID)).OrderBy(mergedomain.ProjectMergeRequestSchema.IID.Desc()).Limit(1)
		last, err := repo.First(ctx, query)
		if err == nil {
			nextIID = last.IID + 1
		} else if err != nil && err != dbxrepo.ErrNotFound {
			return err
		}
		now := time.Now().UTC()
		item := mergedomain.ProjectMergeRequest{
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
		return mergedomain.ProjectMergeRequest{}, err
	}
	return created, nil
}

func (r *Repository) UpdateByID(ctx context.Context, id int64, input UpdateInput) error {
	assignments := make([]querydsl.Assignment, 0, 4)
	if input.Title != nil {
		assignments = append(assignments, mergedomain.ProjectMergeRequestSchema.Title.Set(strings.TrimSpace(*input.Title)))
	}
	if input.Description != nil {
		assignments = append(assignments, mergedomain.ProjectMergeRequestSchema.Description.Set(strings.TrimSpace(*input.Description)))
	}
	if input.State != nil {
		assignments = append(assignments, mergedomain.ProjectMergeRequestSchema.State.Set(strings.TrimSpace(*input.State)))
	}
	assignments = append(assignments, mergedomain.ProjectMergeRequestSchema.UpdatedAt.Set(time.Now().UTC()))
	if _, err := r.base.UpdateByID(ctx, id, assignments...); err != nil {
		return fmt.Errorf("update merge request: %w", err)
	}
	return nil
}
