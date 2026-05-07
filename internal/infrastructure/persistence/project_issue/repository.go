package projectissue

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
	base *dbxrepo.Base[issuedomain.ProjectIssue, dbschema.ProjectIssueSchemaDef]
}

type CreateInput = issueports.CreateProjectIssueInput

type UpdateInput = issueports.UpdateProjectIssueInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[issuedomain.ProjectIssue](db, dbschema.ProjectIssueSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func NewProjectIssueRepository(repo *Repository) issueports.ProjectIssueRepository {
	return repo
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[issuedomain.ProjectIssue], error) {
	query := querydsl.Select(dbschema.ProjectIssueSchema.AllColumns().Values()...).
		From(dbschema.ProjectIssueSchema).
		Where(dbschema.ProjectIssueSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectIssueSchema.IID.Desc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByProjectAndIID(ctx context.Context, projectID int64, iid int64) (issuedomain.ProjectIssue, error) {
	query := querydsl.Select(dbschema.ProjectIssueSchema.AllColumns().Values()...).
		From(dbschema.ProjectIssueSchema).
		Where(querydsl.And(
			dbschema.ProjectIssueSchema.ProjectID.Eq(projectID),
			dbschema.ProjectIssueSchema.IID.Eq(iid),
		)).
		Limit(1)
	return persistence.One(r.base.First(ctx, query))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (issuedomain.ProjectIssue, error) {
	var created issuedomain.ProjectIssue
	err := r.base.InTx(ctx, nil, func(tx *dbx.Tx, repo *dbxrepo.Base[issuedomain.ProjectIssue, dbschema.ProjectIssueSchemaDef]) error {
		nextIID := int64(1)
		query := querydsl.Select(dbschema.ProjectIssueSchema.AllColumns().Values()...).
			From(dbschema.ProjectIssueSchema).
			Where(dbschema.ProjectIssueSchema.ProjectID.Eq(input.ProjectID)).
			OrderBy(dbschema.ProjectIssueSchema.IID.Desc()).
			Limit(1)
		last, err := repo.First(ctx, query)
		if err == nil {
			nextIID = last.IID + 1
		} else if err != nil && !persistence.IsNotFound(err) {
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
		assignments = append(assignments, dbschema.ProjectIssueSchema.Title.Set(strings.TrimSpace(*input.Title)))
	}
	if input.Description != nil {
		assignments = append(assignments, dbschema.ProjectIssueSchema.Description.Set(strings.TrimSpace(*input.Description)))
	}
	if input.State != nil {
		assignments = append(assignments, dbschema.ProjectIssueSchema.State.Set(strings.TrimSpace(*input.State)))
	}
	assignments = append(assignments, dbschema.ProjectIssueSchema.UpdatedAt.Set(time.Now().UTC()))
	if _, err := r.base.UpdateByID(ctx, id, assignments...); err != nil {
		return fmt.Errorf("update project issue: %w", err)
	}
	return nil
}
