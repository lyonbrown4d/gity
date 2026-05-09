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
	"github.com/samber/oops"
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
	return persistence.Many(dbxrepo.Query(r.base).
		Where(dbschema.ProjectMergeRequestSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectMergeRequestSchema.IID.Desc()).
		List(ctx))
}

func (r *Repository) GetByProjectAndIID(ctx context.Context, projectID, iid int64) (mergedomain.ProjectMergeRequest, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectMergeRequestSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectMergeRequestSchema.IID.Eq(iid)).
		First(ctx))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (mergedomain.ProjectMergeRequest, error) {
	var created mergedomain.ProjectMergeRequest
	err := r.base.InTx(ctx, nil, func(_ *dbx.Tx, repo *dbxrepo.Base[mergedomain.ProjectMergeRequest, dbschema.ProjectMergeRequestSchemaDef]) error {
		nextIID := int64(1)
		last, err := dbxrepo.Query(repo).
			Where(dbschema.ProjectMergeRequestSchema.ProjectID.Eq(input.ProjectID)).
			OrderBy(dbschema.ProjectMergeRequestSchema.IID.Desc()).
			First(ctx)
		if err == nil {
			nextIID = last.IID + 1
		} else if !persistence.IsNotFound(err) {
			return oops.In("persistence.merge_request").With("project_id", input.ProjectID).Wrapf(err, "load last merge request")
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
		return mergedomain.ProjectMergeRequest{}, oops.In("persistence.merge_request").With("project_id", input.ProjectID).Wrapf(err, "create merge request")
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
	if _, err := dbxrepo.PatchSet(r.base, projectMergeRequestKey(id)).Set(assignments...).Apply(ctx); err != nil {
		return fmt.Errorf("update merge request: %w", err)
	}
	return nil
}

func projectMergeRequestKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectMergeRequestSchema.ID, id))
}
