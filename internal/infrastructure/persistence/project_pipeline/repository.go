package projectpipeline

import (
	"context"
	"fmt"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	ciports "github.com/lyonbrown4d/gity/internal/application/ports"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	persistence "github.com/lyonbrown4d/gity/internal/infrastructure/persistence"
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
	projectcounter "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_counter"
	"github.com/samber/oops"
	"strings"
	"time"
)

const (
	StatusPending   = ciports.ProjectPipelineStatusPending
	StatusRunning   = ciports.ProjectPipelineStatusRunning
	StatusSucceeded = ciports.ProjectPipelineStatusSucceeded
	StatusFailed    = ciports.ProjectPipelineStatusFailed
	StatusCancelled = ciports.ProjectPipelineStatusCancelled
)

type Repository struct {
	base *dbxrepo.Base[cidomain.ProjectPipeline, dbschema.ProjectPipelineSchemaDef]
}

type CreateInput = ciports.CreateProjectPipelineInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[cidomain.ProjectPipeline](db, dbschema.ProjectPipelineSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectPipelineRepository(repo *Repository) ciports.ProjectPipelineRepository {
	return repo
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[cidomain.ProjectPipeline], error) {
	return persistence.Many(dbxrepo.Query(r.base).
		Where(dbschema.ProjectPipelineSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectPipelineSchema.IID.Desc()).
		List(ctx))
}

func (r *Repository) GetByProjectAndID(ctx context.Context, projectID, id int64) (cidomain.ProjectPipeline, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectPipelineSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectPipelineSchema.ID.Eq(id)).
		First(ctx))
}

func (r *Repository) GetByProjectSourceRefCommit(ctx context.Context, projectID int64, source, refName, commitSHA string) (cidomain.ProjectPipeline, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectPipelineSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectPipelineSchema.Source.Eq(strings.TrimSpace(source))).
		Where(dbschema.ProjectPipelineSchema.RefName.Eq(strings.TrimSpace(refName))).
		Where(dbschema.ProjectPipelineSchema.CommitSHA.Eq(strings.TrimSpace(commitSHA))).
		OrderBy(dbschema.ProjectPipelineSchema.IID.Desc()).
		First(ctx))
}

func (r *Repository) GetLatestByProjectRefCommit(ctx context.Context, projectID int64, refName, commitSHA string) (cidomain.ProjectPipeline, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectPipelineSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectPipelineSchema.RefName.Eq(strings.TrimSpace(refName))).
		Where(dbschema.ProjectPipelineSchema.CommitSHA.Eq(strings.TrimSpace(commitSHA))).
		OrderBy(dbschema.ProjectPipelineSchema.IID.Desc()).
		First(ctx))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (cidomain.ProjectPipeline, error) {
	var created cidomain.ProjectPipeline
	err := r.base.InTx(ctx, nil, func(tx *dbx.Tx, repo *dbxrepo.Base[cidomain.ProjectPipeline, dbschema.ProjectPipelineSchemaDef]) error {
		nextIID, err := projectcounter.Next(ctx, tx, input.ProjectID, projectcounter.CounterPipeline)
		if err != nil {
			return oops.In("persistence.pipeline").With("project_id", input.ProjectID).Wrapf(err, "allocate pipeline iid")
		}
		item := newProjectPipeline(input, nextIID)
		if err := repo.Create(ctx, &item); err != nil {
			return fmt.Errorf("insert project pipeline: %w", err)
		}
		created = item
		return nil
	})
	if err != nil {
		return cidomain.ProjectPipeline{}, oops.In("persistence.pipeline").With("project_id", input.ProjectID).Wrapf(err, "create pipeline")
	}
	return created, nil
}

func newProjectPipeline(input CreateInput, iid int64) cidomain.ProjectPipeline {
	now := time.Now().UTC()
	return cidomain.ProjectPipeline{
		ProjectID:     input.ProjectID,
		IID:           iid,
		Name:          strings.TrimSpace(input.Name),
		Source:        defaultPipelineSource(input.Source),
		RefName:       strings.TrimSpace(input.RefName),
		CommitSHA:     strings.TrimSpace(input.CommitSHA),
		Status:        defaultPipelineStatus(input.Status),
		ConfigSource:  defaultPipelineConfigSource(input.ConfigSource),
		ConfigContent: strings.TrimSpace(input.ConfigContent),
		CreatedAt:     now,
		UpdatedAt:     now,
	}
}

func defaultPipelineStatus(value string) string {
	status := strings.TrimSpace(value)
	if status == "" {
		return StatusPending
	}
	return status
}

func defaultPipelineSource(value string) string {
	source := strings.TrimSpace(value)
	if source == "" {
		return "api"
	}
	return source
}

func defaultPipelineConfigSource(value string) string {
	configSource := strings.TrimSpace(value)
	if configSource == "" {
		return ".gity-ci.plano"
	}
	return configSource
}

func (r *Repository) UpdateStatus(ctx context.Context, item cidomain.ProjectPipeline, status string) error {
	status = strings.TrimSpace(status)
	if status == "" || item.Status == status {
		return nil
	}
	now := time.Now().UTC()
	assignments := []querydsl.Assignment{
		dbschema.ProjectPipelineSchema.Status.Set(status),
		dbschema.ProjectPipelineSchema.UpdatedAt.Set(now),
	}
	if status == StatusRunning && item.StartedAt.IsZero() {
		assignments = append(assignments, dbschema.ProjectPipelineSchema.StartedAt.Set(now))
	}
	if isTerminalStatus(status) && item.FinishedAt.IsZero() {
		if item.StartedAt.IsZero() {
			assignments = append(assignments, dbschema.ProjectPipelineSchema.StartedAt.Set(now))
		}
		assignments = append(assignments, dbschema.ProjectPipelineSchema.FinishedAt.Set(now))
	}
	if _, err := dbxrepo.PatchSet(r.base, projectPipelineKey(item.ID)).Set(assignments...).Apply(ctx); err != nil {
		return fmt.Errorf("update project pipeline status: %w", err)
	}
	return nil
}

func projectPipelineKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectPipelineSchema.ID, id))
}

func isTerminalStatus(status string) bool {
	switch status {
	case StatusSucceeded, StatusFailed, StatusCancelled:
		return true
	default:
		return false
	}
}
