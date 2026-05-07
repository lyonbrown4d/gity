package projectrunner

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
	StatusOnline  = "online"
	StatusOffline = "offline"
)

type Repository struct {
	base *dbxrepo.Base[cidomain.ProjectRunner, cidomain.ProjectRunnerSchemaDef]
}

type CreateInput struct {
	ProjectID   int64
	Name        string
	Description string
	Tags        string
	Token       string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[cidomain.ProjectRunner](db, cidomain.ProjectRunnerSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[cidomain.ProjectRunner], error) {
	query := querydsl.Select(cidomain.ProjectRunnerSchema.AllColumns().Values()...).
		From(cidomain.ProjectRunnerSchema).
		Where(cidomain.ProjectRunnerSchema.ProjectID.Eq(projectID)).
		OrderBy(cidomain.ProjectRunnerSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByProjectAndID(ctx context.Context, projectID int64, id int64) (cidomain.ProjectRunner, error) {
	query := querydsl.Select(cidomain.ProjectRunnerSchema.AllColumns().Values()...).
		From(cidomain.ProjectRunnerSchema).
		Where(querydsl.And(
			cidomain.ProjectRunnerSchema.ProjectID.Eq(projectID),
			cidomain.ProjectRunnerSchema.ID.Eq(id),
		)).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) GetByToken(ctx context.Context, token string) (cidomain.ProjectRunner, error) {
	return r.base.GetByKey(ctx, dbxrepo.Key{
		"token": strings.TrimSpace(token),
	})
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (cidomain.ProjectRunner, error) {
	now := time.Now().UTC()
	item := cidomain.ProjectRunner{
		ProjectID:     input.ProjectID,
		Name:          strings.TrimSpace(input.Name),
		Description:   strings.TrimSpace(input.Description),
		Tags:          normalizeTags(input.Tags),
		Token:         strings.TrimSpace(input.Token),
		Status:        StatusOffline,
		Active:        1,
		LastContactAt: time.Time{},
		CreatedAt:     now,
		UpdatedAt:     now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return cidomain.ProjectRunner{}, fmt.Errorf("insert project runner: %w", err)
	}
	return item, nil
}

func (r *Repository) MarkHeartbeat(ctx context.Context, id int64) error {
	now := time.Now().UTC()
	if _, err := r.base.UpdateByID(ctx, id,
		cidomain.ProjectRunnerSchema.Status.Set(StatusOnline),
		cidomain.ProjectRunnerSchema.LastContactAt.Set(now),
		cidomain.ProjectRunnerSchema.UpdatedAt.Set(now),
	); err != nil {
		return fmt.Errorf("record project runner heartbeat: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := r.base.DeleteByID(ctx, id); err != nil {
		return fmt.Errorf("delete project runner: %w", err)
	}
	return nil
}

func normalizeTags(value string) string {
	parts := strings.Split(value, ",")
	seen := map[string]struct{}{}
	out := make([]string, 0, len(parts))
	for _, part := range parts {
		trimmed := strings.TrimSpace(strings.ToLower(part))
		if trimmed == "" {
			continue
		}
		if _, ok := seen[trimmed]; ok {
			continue
		}
		seen[trimmed] = struct{}{}
		out = append(out, trimmed)
	}
	return strings.Join(out, ",")
}
