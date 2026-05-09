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
	"github.com/samber/oops"
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
		base: dbxrepo.NewWithOptions[issuedomain.ProjectIssue](db, dbschema.ProjectIssueSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectIssueRepository(repo *Repository) issueports.ProjectIssueRepository {
	return repo
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[issuedomain.ProjectIssue], error) {
	return persistence.Many(dbxrepo.Query(r.base).
		Where(dbschema.ProjectIssueSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectIssueSchema.IID.Desc()).
		List(ctx))
}

func (r *Repository) GetByProjectAndIID(ctx context.Context, projectID, iid int64) (issuedomain.ProjectIssue, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectIssueSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectIssueSchema.IID.Eq(iid)).
		First(ctx))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (issuedomain.ProjectIssue, error) {
	var created issuedomain.ProjectIssue
	err := r.base.InTx(ctx, nil, func(tx *dbx.Tx, repo *dbxrepo.Base[issuedomain.ProjectIssue, dbschema.ProjectIssueSchemaDef]) error {
		nextIID := int64(1)
		last, err := dbxrepo.Query(repo).
			Where(dbschema.ProjectIssueSchema.ProjectID.Eq(input.ProjectID)).
			OrderBy(dbschema.ProjectIssueSchema.IID.Desc()).
			First(ctx)
		if err == nil {
			nextIID = last.IID + 1
		} else if !persistence.IsNotFound(err) {
			return oops.In("persistence.project_issue").With("project_id", input.ProjectID).Wrapf(err, "load last project issue")
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
		return issuedomain.ProjectIssue{}, oops.In("persistence.project_issue").With("project_id", input.ProjectID).Wrapf(err, "create project issue")
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
	if _, err := dbxrepo.PatchSet(r.base, projectIssueKey(id)).Set(assignments...).Apply(ctx); err != nil {
		return fmt.Errorf("update project issue: %w", err)
	}
	return nil
}

func projectIssueKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectIssueSchema.ID, id))
}
