package projectpipelinejob

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
	base *dbxrepo.Base[entity.ProjectPipelineJob, entity.ProjectPipelineJobSchemaDef]
}

type CreateInput struct {
	ProjectID    int64
	PipelineID   int64
	ProjectJobID int64
	Name         string
	Stage        string
	Needs        string
	Image        string
	Script       string
	Artifacts    string
	SortOrder    int
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[entity.ProjectPipelineJob](db, entity.ProjectPipelineJobSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByPipelineID(ctx context.Context, projectID int64, pipelineID int64) (*collectionx.List[entity.ProjectPipelineJob], error) {
	query := dbx.Select(entity.ProjectPipelineJobSchema.AllColumns().Values()...).
		From(entity.ProjectPipelineJobSchema).
		Where(dbx.And(
			entity.ProjectPipelineJobSchema.ProjectID.Eq(projectID),
			entity.ProjectPipelineJobSchema.PipelineID.Eq(pipelineID),
		)).
		OrderBy(entity.ProjectPipelineJobSchema.SortOrder.Asc(), entity.ProjectPipelineJobSchema.ID.Asc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByProjectJobID(ctx context.Context, projectID int64, projectJobID int64) (entity.ProjectPipelineJob, error) {
	query := dbx.Select(entity.ProjectPipelineJobSchema.AllColumns().Values()...).
		From(entity.ProjectPipelineJobSchema).
		Where(dbx.And(
			entity.ProjectPipelineJobSchema.ProjectID.Eq(projectID),
			entity.ProjectPipelineJobSchema.ProjectJobID.Eq(projectJobID),
		)).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.ProjectPipelineJob, error) {
	now := time.Now().UTC()
	item := entity.ProjectPipelineJob{
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
		return entity.ProjectPipelineJob{}, fmt.Errorf("insert project pipeline job: %w", err)
	}
	return item, nil
}
