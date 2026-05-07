package projectpipeline

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

const (
	StatusPending   = "pending"
	StatusRunning   = "running"
	StatusSucceeded = "succeeded"
	StatusFailed    = "failed"
	StatusCancelled = "cancelled"
)

type Repository struct {
	base *dbxrepo.Base[entity.ProjectPipeline, entity.ProjectPipelineSchemaDef]
}

type CreateInput struct {
	ProjectID     int64
	Name          string
	Source        string
	RefName       string
	CommitSHA     string
	Status        string
	ConfigSource  string
	ConfigContent string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[entity.ProjectPipeline](db, entity.ProjectPipelineSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[entity.ProjectPipeline], error) {
	query := dbx.Select(entity.ProjectPipelineSchema.AllColumns().Values()...).
		From(entity.ProjectPipelineSchema).
		Where(entity.ProjectPipelineSchema.ProjectID.Eq(projectID)).
		OrderBy(entity.ProjectPipelineSchema.IID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByProjectAndID(ctx context.Context, projectID int64, id int64) (entity.ProjectPipeline, error) {
	query := dbx.Select(entity.ProjectPipelineSchema.AllColumns().Values()...).
		From(entity.ProjectPipelineSchema).
		Where(dbx.And(
			entity.ProjectPipelineSchema.ProjectID.Eq(projectID),
			entity.ProjectPipelineSchema.ID.Eq(id),
		)).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) GetByProjectSourceRefCommit(ctx context.Context, projectID int64, source string, refName string, commitSHA string) (entity.ProjectPipeline, error) {
	query := dbx.Select(entity.ProjectPipelineSchema.AllColumns().Values()...).
		From(entity.ProjectPipelineSchema).
		Where(dbx.And(
			entity.ProjectPipelineSchema.ProjectID.Eq(projectID),
			entity.ProjectPipelineSchema.Source.Eq(strings.TrimSpace(source)),
			entity.ProjectPipelineSchema.RefName.Eq(strings.TrimSpace(refName)),
			entity.ProjectPipelineSchema.CommitSHA.Eq(strings.TrimSpace(commitSHA)),
		)).
		OrderBy(entity.ProjectPipelineSchema.IID.Desc()).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) GetLatestByProjectRefCommit(ctx context.Context, projectID int64, refName string, commitSHA string) (entity.ProjectPipeline, error) {
	query := dbx.Select(entity.ProjectPipelineSchema.AllColumns().Values()...).
		From(entity.ProjectPipelineSchema).
		Where(dbx.And(
			entity.ProjectPipelineSchema.ProjectID.Eq(projectID),
			entity.ProjectPipelineSchema.RefName.Eq(strings.TrimSpace(refName)),
			entity.ProjectPipelineSchema.CommitSHA.Eq(strings.TrimSpace(commitSHA)),
		)).
		OrderBy(entity.ProjectPipelineSchema.IID.Desc()).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.ProjectPipeline, error) {
	var created entity.ProjectPipeline
	err := r.base.InTx(ctx, nil, func(_ *dbx.Tx, repo *dbxrepo.Base[entity.ProjectPipeline, entity.ProjectPipelineSchemaDef]) error {
		nextIID := int64(1)
		query := dbx.Select(entity.ProjectPipelineSchema.AllColumns().Values()...).
			From(entity.ProjectPipelineSchema).
			Where(entity.ProjectPipelineSchema.ProjectID.Eq(input.ProjectID)).
			OrderBy(entity.ProjectPipelineSchema.IID.Desc()).
			Limit(1)
		last, err := repo.First(ctx, query)
		if err == nil {
			nextIID = last.IID + 1
		} else if err != nil && err != dbxrepo.ErrNotFound {
			return err
		}
		status := strings.TrimSpace(input.Status)
		if status == "" {
			status = StatusPending
		}
		source := strings.TrimSpace(input.Source)
		if source == "" {
			source = "api"
		}
		configSource := strings.TrimSpace(input.ConfigSource)
		if configSource == "" {
			configSource = ".gity-ci.plano"
		}
		now := time.Now().UTC()
		item := entity.ProjectPipeline{
			ProjectID:     input.ProjectID,
			IID:           nextIID,
			Name:          strings.TrimSpace(input.Name),
			Source:        source,
			RefName:       strings.TrimSpace(input.RefName),
			CommitSHA:     strings.TrimSpace(input.CommitSHA),
			Status:        status,
			ConfigSource:  configSource,
			ConfigContent: strings.TrimSpace(input.ConfigContent),
			CreatedAt:     now,
			UpdatedAt:     now,
		}
		if err := repo.Create(ctx, &item); err != nil {
			return fmt.Errorf("insert project pipeline: %w", err)
		}
		created = item
		return nil
	})
	if err != nil {
		return entity.ProjectPipeline{}, err
	}
	return created, nil
}

func (r *Repository) UpdateStatus(ctx context.Context, item entity.ProjectPipeline, status string) error {
	status = strings.TrimSpace(status)
	if status == "" || item.Status == status {
		return nil
	}
	now := time.Now().UTC()
	assignments := []dbx.Assignment{
		entity.ProjectPipelineSchema.Status.Set(status),
		entity.ProjectPipelineSchema.UpdatedAt.Set(now),
	}
	if status == StatusRunning && item.StartedAt.IsZero() {
		assignments = append(assignments, entity.ProjectPipelineSchema.StartedAt.Set(now))
	}
	if isTerminalStatus(status) && item.FinishedAt.IsZero() {
		if item.StartedAt.IsZero() {
			assignments = append(assignments, entity.ProjectPipelineSchema.StartedAt.Set(now))
		}
		assignments = append(assignments, entity.ProjectPipelineSchema.FinishedAt.Set(now))
	}
	if _, err := r.base.UpdateByID(ctx, item.ID, assignments...); err != nil {
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
