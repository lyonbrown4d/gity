package projectpipelinejob

import (
	"context"
	"fmt"
	ciports "github.com/DaiYuANg/gity/internal/application/ports"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
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
	base *dbxrepo.Base[cidomain.ProjectPipelineJob, dbschema.ProjectPipelineJobSchemaDef]
}

type CreateInput = ciports.CreateProjectPipelineJobInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[cidomain.ProjectPipelineJob](db, dbschema.ProjectPipelineJobSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectPipelineJobRepository(repo *Repository) ciports.ProjectPipelineJobRepository {
	return repo
}

func (r *Repository) ListByPipelineID(ctx context.Context, projectID int64, pipelineID int64) (*collectionx.List[cidomain.ProjectPipelineJob], error) {
	query := querydsl.Select(dbschema.ProjectPipelineJobSchema.AllColumns().Values()...).
		From(dbschema.ProjectPipelineJobSchema).
		Where(querydsl.And(
			dbschema.ProjectPipelineJobSchema.ProjectID.Eq(projectID),
			dbschema.ProjectPipelineJobSchema.PipelineID.Eq(pipelineID),
		)).
		OrderBy(dbschema.ProjectPipelineJobSchema.SortOrder.Asc(), dbschema.ProjectPipelineJobSchema.ID.Asc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByProjectJobID(ctx context.Context, projectID int64, projectJobID int64) (cidomain.ProjectPipelineJob, error) {
	query := querydsl.Select(dbschema.ProjectPipelineJobSchema.AllColumns().Values()...).
		From(dbschema.ProjectPipelineJobSchema).
		Where(querydsl.And(
			dbschema.ProjectPipelineJobSchema.ProjectID.Eq(projectID),
			dbschema.ProjectPipelineJobSchema.ProjectJobID.Eq(projectJobID),
		)).
		Limit(1)
	return persistence.One(r.base.First(ctx, query))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (cidomain.ProjectPipelineJob, error) {
	now := time.Now().UTC()
	item := cidomain.ProjectPipelineJob{
		ProjectID:    input.ProjectID,
		PipelineID:   input.PipelineID,
		ProjectJobID: input.ProjectJobID,
		Name:         strings.TrimSpace(input.Name),
		Stage:        strings.TrimSpace(input.Stage),
		Needs:        strings.TrimSpace(input.Needs),
		Image:        strings.TrimSpace(input.Image),
		Script:       strings.TrimSpace(input.Script),
		Artifacts:    strings.TrimSpace(input.Artifacts),
		SortOrder:    input.SortOrder,
		CreatedAt:    now,
		UpdatedAt:    now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return cidomain.ProjectPipelineJob{}, fmt.Errorf("insert project pipeline job: %w", err)
	}
	return item, nil
}
