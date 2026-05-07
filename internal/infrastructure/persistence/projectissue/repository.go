package projectissue

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
	base *dbxrepo.Base[issuedomain.ProjectIssue, issuedomain.ProjectIssueSchemaDef]
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
		base: dbxrepo.NewWithOptions[issuedomain.ProjectIssue](db, issuedomain.ProjectIssueSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[issuedomain.ProjectIssue], error) {
	query := querydsl.Select(issuedomain.ProjectIssueSchema.AllColumns().Values()...).
		From(issuedomain.ProjectIssueSchema).
		Where(issuedomain.ProjectIssueSchema.ProjectID.Eq(projectID)).
		OrderBy(issuedomain.ProjectIssueSchema.IID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByProjectAndIID(ctx context.Context, projectID int64, iid int64) (issuedomain.ProjectIssue, error) {
	query := querydsl.Select(issuedomain.ProjectIssueSchema.AllColumns().Values()...).
		From(issuedomain.ProjectIssueSchema).
		Where(querydsl.And(
			issuedomain.ProjectIssueSchema.ProjectID.Eq(projectID),
			issuedomain.ProjectIssueSchema.IID.Eq(iid),
		)).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (issuedomain.ProjectIssue, error) {
	var created issuedomain.ProjectIssue
	err := r.base.InTx(ctx, nil, func(tx *dbx.Tx, repo *dbxrepo.Base[issuedomain.ProjectIssue, issuedomain.ProjectIssueSchemaDef]) error {
		nextIID := int64(1)
		query := querydsl.Select(issuedomain.ProjectIssueSchema.AllColumns().Values()...).
			From(issuedomain.ProjectIssueSchema).
			Where(issuedomain.ProjectIssueSchema.ProjectID.Eq(input.ProjectID)).
			OrderBy(issuedomain.ProjectIssueSchema.IID.Desc()).
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
		item := issuedomain.ProjectIssue{
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
		return issuedomain.ProjectIssue{}, err
	}
	return created, nil
}

func (r *Repository) UpdateByID(ctx context.Context, id int64, input UpdateInput) error {
	assignments := make([]querydsl.Assignment, 0, 4)
	if input.Title != nil {
		assignments = append(assignments, issuedomain.ProjectIssueSchema.Title.Set(strings.TrimSpace(*input.Title)))
	}
	if input.Description != nil {
		assignments = append(assignments, issuedomain.ProjectIssueSchema.Description.Set(strings.TrimSpace(*input.Description)))
	}
	if input.State != nil {
		assignments = append(assignments, issuedomain.ProjectIssueSchema.State.Set(strings.TrimSpace(*input.State)))
	}
	assignments = append(assignments, issuedomain.ProjectIssueSchema.UpdatedAt.Set(time.Now().UTC()))
	if _, err := r.base.UpdateByID(ctx, id, assignments...); err != nil {
		return fmt.Errorf("update project issue: %w", err)
	}
	return nil
}
