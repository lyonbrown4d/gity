package projectpipeline

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
	query := querydsl.Select(dbschema.ProjectPipelineSchema.AllColumns().Values()...).
		From(dbschema.ProjectPipelineSchema).
		Where(dbschema.ProjectPipelineSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectPipelineSchema.IID.Desc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByProjectAndID(ctx context.Context, projectID, id int64) (cidomain.ProjectPipeline, error) {
	query := querydsl.Select(dbschema.ProjectPipelineSchema.AllColumns().Values()...).
		From(dbschema.ProjectPipelineSchema).
		Where(querydsl.And(
			dbschema.ProjectPipelineSchema.ProjectID.Eq(projectID),
			dbschema.ProjectPipelineSchema.ID.Eq(id),
		)).
		Limit(1)
	return persistence.One(r.base.First(ctx, query))
}

func (r *Repository) GetByProjectSourceRefCommit(ctx context.Context, projectID int64, source, refName, commitSHA string) (cidomain.ProjectPipeline, error) {
	query := querydsl.Select(dbschema.ProjectPipelineSchema.AllColumns().Values()...).
		From(dbschema.ProjectPipelineSchema).
		Where(querydsl.And(
			dbschema.ProjectPipelineSchema.ProjectID.Eq(projectID),
			dbschema.ProjectPipelineSchema.Source.Eq(strings.TrimSpace(source)),
			dbschema.ProjectPipelineSchema.RefName.Eq(strings.TrimSpace(refName)),
			dbschema.ProjectPipelineSchema.CommitSHA.Eq(strings.TrimSpace(commitSHA)),
		)).
		OrderBy(dbschema.ProjectPipelineSchema.IID.Desc()).
		Limit(1)
	return persistence.One(r.base.First(ctx, query))
}

func (r *Repository) GetLatestByProjectRefCommit(ctx context.Context, projectID int64, refName, commitSHA string) (cidomain.ProjectPipeline, error) {
	query := querydsl.Select(dbschema.ProjectPipelineSchema.AllColumns().Values()...).
		From(dbschema.ProjectPipelineSchema).
		Where(querydsl.And(
			dbschema.ProjectPipelineSchema.ProjectID.Eq(projectID),
			dbschema.ProjectPipelineSchema.RefName.Eq(strings.TrimSpace(refName)),
			dbschema.ProjectPipelineSchema.CommitSHA.Eq(strings.TrimSpace(commitSHA)),
		)).
		OrderBy(dbschema.ProjectPipelineSchema.IID.Desc()).
		Limit(1)
	return persistence.One(r.base.First(ctx, query))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (cidomain.ProjectPipeline, error) {
	var created cidomain.ProjectPipeline
	err := r.base.InTx(ctx, nil, func(_ *dbx.Tx, repo *dbxrepo.Base[cidomain.ProjectPipeline, dbschema.ProjectPipelineSchemaDef]) error {
		nextIID, err := nextPipelineIID(ctx, repo, input.ProjectID)
		if err != nil {
			return err
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

func nextPipelineIID(ctx context.Context, repo *dbxrepo.Base[cidomain.ProjectPipeline, dbschema.ProjectPipelineSchemaDef], projectID int64) (int64, error) {
	query := querydsl.Select(dbschema.ProjectPipelineSchema.AllColumns().Values()...).
		From(dbschema.ProjectPipelineSchema).
		Where(dbschema.ProjectPipelineSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectPipelineSchema.IID.Desc()).
		Limit(1)
	last, err := repo.First(ctx, query)
	if err == nil {
		return last.IID + 1, nil
	}
	if persistence.IsNotFound(err) {
		return 1, nil
	}
	return 0, oops.In("persistence.pipeline").With("project_id", projectID).Wrapf(err, "load last pipeline")
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
	if _, err := dbxrepo.By(r.base, dbschema.ProjectPipelineSchema.ID).Update(ctx, item.ID, assignments...); err != nil {
		return fmt.Errorf("update project pipeline status: %w", err)
	}
	return nil
}

func isTerminalStatus(status string) bool {
	switch status {
	case StatusSucceeded, StatusFailed, StatusCancelled:
		return true
	default:
		return false
	}
}
