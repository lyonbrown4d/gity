package projectpipelinejob

import (
	"context"
	"fmt"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[cidomain.ProjectPipelineJob, cidomain.ProjectPipelineJobSchemaDef]
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
		base: dbxrepo.NewWithOptions[cidomain.ProjectPipelineJob](db, cidomain.ProjectPipelineJobSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByPipelineID(ctx context.Context, projectID int64, pipelineID int64) (*collectionx.List[cidomain.ProjectPipelineJob], error) {
	query := querydsl.Select(cidomain.ProjectPipelineJobSchema.AllColumns().Values()...).
		From(cidomain.ProjectPipelineJobSchema).
		Where(querydsl.And(
			cidomain.ProjectPipelineJobSchema.ProjectID.Eq(projectID),
			cidomain.ProjectPipelineJobSchema.PipelineID.Eq(pipelineID),
		)).
		OrderBy(cidomain.ProjectPipelineJobSchema.SortOrder.Asc(), cidomain.ProjectPipelineJobSchema.ID.Asc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByProjectJobID(ctx context.Context, projectID int64, projectJobID int64) (cidomain.ProjectPipelineJob, error) {
	query := querydsl.Select(cidomain.ProjectPipelineJobSchema.AllColumns().Values()...).
		From(cidomain.ProjectPipelineJobSchema).
		Where(querydsl.And(
			cidomain.ProjectPipelineJobSchema.ProjectID.Eq(projectID),
			cidomain.ProjectPipelineJobSchema.ProjectJobID.Eq(projectJobID),
		)).
		Limit(1)
	return r.base.First(ctx, query)
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
