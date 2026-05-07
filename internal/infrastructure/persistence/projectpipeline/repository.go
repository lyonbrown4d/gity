package projectpipeline

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

const (
	StatusPending   = "pending"
	StatusRunning   = "running"
	StatusSucceeded = "succeeded"
	StatusFailed    = "failed"
	StatusCancelled = "cancelled"
)

type Repository struct {
	base *dbxrepo.Base[cidomain.ProjectPipeline, cidomain.ProjectPipelineSchemaDef]
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
		base: dbxrepo.NewWithOptions[cidomain.ProjectPipeline](db, cidomain.ProjectPipelineSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[cidomain.ProjectPipeline], error) {
	query := querydsl.Select(cidomain.ProjectPipelineSchema.AllColumns().Values()...).
		From(cidomain.ProjectPipelineSchema).
		Where(cidomain.ProjectPipelineSchema.ProjectID.Eq(projectID)).
		OrderBy(cidomain.ProjectPipelineSchema.IID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByProjectAndID(ctx context.Context, projectID int64, id int64) (cidomain.ProjectPipeline, error) {
	query := querydsl.Select(cidomain.ProjectPipelineSchema.AllColumns().Values()...).
		From(cidomain.ProjectPipelineSchema).
		Where(querydsl.And(
			cidomain.ProjectPipelineSchema.ProjectID.Eq(projectID),
			cidomain.ProjectPipelineSchema.ID.Eq(id),
		)).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) GetByProjectSourceRefCommit(ctx context.Context, projectID int64, source string, refName string, commitSHA string) (cidomain.ProjectPipeline, error) {
	query := querydsl.Select(cidomain.ProjectPipelineSchema.AllColumns().Values()...).
		From(cidomain.ProjectPipelineSchema).
		Where(querydsl.And(
			cidomain.ProjectPipelineSchema.ProjectID.Eq(projectID),
			cidomain.ProjectPipelineSchema.Source.Eq(strings.TrimSpace(source)),
			cidomain.ProjectPipelineSchema.RefName.Eq(strings.TrimSpace(refName)),
			cidomain.ProjectPipelineSchema.CommitSHA.Eq(strings.TrimSpace(commitSHA)),
		)).
		OrderBy(cidomain.ProjectPipelineSchema.IID.Desc()).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) GetLatestByProjectRefCommit(ctx context.Context, projectID int64, refName string, commitSHA string) (cidomain.ProjectPipeline, error) {
	query := querydsl.Select(cidomain.ProjectPipelineSchema.AllColumns().Values()...).
		From(cidomain.ProjectPipelineSchema).
		Where(querydsl.And(
			cidomain.ProjectPipelineSchema.ProjectID.Eq(projectID),
			cidomain.ProjectPipelineSchema.RefName.Eq(strings.TrimSpace(refName)),
			cidomain.ProjectPipelineSchema.CommitSHA.Eq(strings.TrimSpace(commitSHA)),
		)).
		OrderBy(cidomain.ProjectPipelineSchema.IID.Desc()).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (cidomain.ProjectPipeline, error) {
	var created cidomain.ProjectPipeline
	err := r.base.InTx(ctx, nil, func(_ *dbx.Tx, repo *dbxrepo.Base[cidomain.ProjectPipeline, cidomain.ProjectPipelineSchemaDef]) error {
		nextIID := int64(1)
		query := querydsl.Select(cidomain.ProjectPipelineSchema.AllColumns().Values()...).
			From(cidomain.ProjectPipelineSchema).
			Where(cidomain.ProjectPipelineSchema.ProjectID.Eq(input.ProjectID)).
			OrderBy(cidomain.ProjectPipelineSchema.IID.Desc()).
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
		item := cidomain.ProjectPipeline{
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
		return cidomain.ProjectPipeline{}, err
	}
	return created, nil
}

func (r *Repository) UpdateStatus(ctx context.Context, item cidomain.ProjectPipeline, status string) error {
	status = strings.TrimSpace(status)
	if status == "" || item.Status == status {
		return nil
	}
	now := time.Now().UTC()
	assignments := []querydsl.Assignment{
		cidomain.ProjectPipelineSchema.Status.Set(status),
		cidomain.ProjectPipelineSchema.UpdatedAt.Set(now),
	}
	if status == StatusRunning && item.StartedAt.IsZero() {
		assignments = append(assignments, cidomain.ProjectPipelineSchema.StartedAt.Set(now))
	}
	if isTerminalStatus(status) && item.FinishedAt.IsZero() {
		if item.StartedAt.IsZero() {
			assignments = append(assignments, cidomain.ProjectPipelineSchema.StartedAt.Set(now))
		}
		assignments = append(assignments, cidomain.ProjectPipelineSchema.FinishedAt.Set(now))
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
