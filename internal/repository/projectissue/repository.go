package projectissue

import (
	"context"
	"fmt"
	"strings"
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
	"github.com/DaiYuANg/gity/internal/entity"
	collectionx "github.com/arcgolabs/collectionx/list"
	dbxrepo "github.com/arcgolabs/dbx/repository"
)

type Repository struct {
	base *dbxrepo.Base[entity.ProjectIssue, entity.ProjectIssueSchemaDef]
}

type CreateInput struct {
	ProjectID    int64
	AuthorUserID int64
	Title        string
	Description  string
	State        string
}

type UpdateInput struct {
	Title       *string
	Description *string
	State       *string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[entity.ProjectIssue](db, entity.ProjectIssueSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[entity.ProjectIssue], error) {
	query := dbx.Select(entity.ProjectIssueSchema.AllColumns().Values()...).
		From(entity.ProjectIssueSchema).
		Where(entity.ProjectIssueSchema.ProjectID.Eq(projectID)).
		OrderBy(entity.ProjectIssueSchema.IID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByProjectAndIID(ctx context.Context, projectID int64, iid int64) (entity.ProjectIssue, error) {
	query := dbx.Select(entity.ProjectIssueSchema.AllColumns().Values()...).
		From(entity.ProjectIssueSchema).
		Where(dbx.And(
			entity.ProjectIssueSchema.ProjectID.Eq(projectID),
			entity.ProjectIssueSchema.IID.Eq(iid),
		)).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.ProjectIssue, error) {
	var created entity.ProjectIssue
	err := r.base.InTx(ctx, nil, func(tx *dbx.Tx, repo *dbxrepo.Base[entity.ProjectIssue, entity.ProjectIssueSchemaDef]) error {
		nextIID := int64(1)
		query := dbx.Select(entity.ProjectIssueSchema.AllColumns().Values()...).
			From(entity.ProjectIssueSchema).
			Where(entity.ProjectIssueSchema.ProjectID.Eq(input.ProjectID)).
			OrderBy(entity.ProjectIssueSchema.IID.Desc()).
			Limit(1)
		last, err := repo.First(ctx, query)
		if err == nil {
			nextIID = last.IID + 1
		} else if err != nil && err != dbxrepo.ErrNotFound {
			return err
		}
		state := strings.TrimSpace(input.State)
		if state == "" {
			state = "opened"
		}
		now := time.Now().UTC()
		item := entity.ProjectIssue{
			ProjectID:    input.ProjectID,
			IID:          nextIID,
			AuthorUserID: input.AuthorUserID,
			Title:        strings.TrimSpace(input.Title),
			Description:  strings.TrimSpace(input.Description),
			State:        state,
			CreatedAt:    now,
			UpdatedAt:    now,
		}
		if err := repo.Create(ctx, &item); err != nil {
			return fmt.Errorf("insert project issue: %w", err)
		}
		created = item
		_ = tx
		return nil
	})
	if err != nil {
		return entity.ProjectIssue{}, err
	}
	return created, nil
}

func (r *Repository) UpdateByID(ctx context.Context, id int64, input UpdateInput) error {
	assignments := make([]dbx.Assignment, 0, 4)
	if input.Title != nil {
		assignments = append(assignments, entity.ProjectIssueSchema.Title.Set(strings.TrimSpace(*input.Title)))
	}
	if input.Description != nil {
		assignments = append(assignments, entity.ProjectIssueSchema.Description.Set(strings.TrimSpace(*input.Description)))
	}
	if input.State != nil {
		assignments = append(assignments, entity.ProjectIssueSchema.State.Set(strings.TrimSpace(*input.State)))
	}
	assignments = append(assignments, entity.ProjectIssueSchema.UpdatedAt.Set(time.Now().UTC()))
	if _, err := r.base.UpdateByID(ctx, id, assignments...); err != nil {
		return fmt.Errorf("update project issue: %w", err)
	}
	return nil
}
