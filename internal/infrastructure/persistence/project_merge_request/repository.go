package projectmergerequest

import (
	"context"
	"fmt"
	mergeports "github.com/DaiYuANg/gity/internal/application/ports"
	mergedomain "github.com/DaiYuANg/gity/internal/domain/merge"
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
	base *dbxrepo.Base[mergedomain.ProjectMergeRequest, dbschema.ProjectMergeRequestSchemaDef]
}

type CreateInput = mergeports.CreateProjectMergeRequestInput

type UpdateInput = mergeports.UpdateProjectMergeRequestInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[mergedomain.ProjectMergeRequest](db, dbschema.ProjectMergeRequestSchema, dbxrepo.WithKeyNotFoundAsError(true))}, nil
}

func NewProjectMergeRequestRepository(repo *Repository) mergeports.ProjectMergeRequestRepository {
	return repo
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[mergedomain.ProjectMergeRequest], error) {
	query := querydsl.Select(dbschema.ProjectMergeRequestSchema.AllColumns().Values()...).From(dbschema.ProjectMergeRequestSchema).Where(dbschema.ProjectMergeRequestSchema.ProjectID.Eq(projectID)).OrderBy(dbschema.ProjectMergeRequestSchema.IID.Desc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByProjectAndIID(ctx context.Context, projectID int64, iid int64) (mergedomain.ProjectMergeRequest, error) {
	query := querydsl.Select(dbschema.ProjectMergeRequestSchema.AllColumns().Values()...).From(dbschema.ProjectMergeRequestSchema).Where(querydsl.And(dbschema.ProjectMergeRequestSchema.ProjectID.Eq(projectID), dbschema.ProjectMergeRequestSchema.IID.Eq(iid))).Limit(1)
	return persistence.One(r.base.First(ctx, query))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (mergedomain.ProjectMergeRequest, error) {
	var created mergedomain.ProjectMergeRequest
	err := r.base.InTx(ctx, nil, func(_ *dbx.Tx, repo *dbxrepo.Base[mergedomain.ProjectMergeRequest, dbschema.ProjectMergeRequestSchemaDef]) error {
		nextIID := int64(1)
		query := querydsl.Select(dbschema.ProjectMergeRequestSchema.AllColumns().Values()...).From(dbschema.ProjectMergeRequestSchema).Where(dbschema.ProjectMergeRequestSchema.ProjectID.Eq(input.ProjectID)).OrderBy(dbschema.ProjectMergeRequestSchema.IID.Desc()).Limit(1)
		last, err := repo.First(ctx, query)
		if err == nil {
			nextIID = last.IID + 1
		} else if !persistence.IsNotFound(err) {
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
		assignments = append(assignments, dbschema.ProjectMergeRequestSchema.Title.Set(strings.TrimSpace(*input.Title)))
	}
	if input.Description != nil {
		assignments = append(assignments, dbschema.ProjectMergeRequestSchema.Description.Set(strings.TrimSpace(*input.Description)))
	}
	if input.State != nil {
		assignments = append(assignments, dbschema.ProjectMergeRequestSchema.State.Set(strings.TrimSpace(*input.State)))
	}
	assignments = append(assignments, dbschema.ProjectMergeRequestSchema.UpdatedAt.Set(time.Now().UTC()))
	if _, err := dbxrepo.By(r.base, dbschema.ProjectMergeRequestSchema.ID).Update(ctx, id, assignments...); err != nil {
		return fmt.Errorf("update merge request: %w", err)
	}
	return nil
}
