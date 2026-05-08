package projectrunner

import (
	"context"
	"fmt"
	ciports "github.com/DaiYuANg/gity/internal/application/ports"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	persistence "github.com/DaiYuANg/gity/internal/infrastructure/persistence"
	dbschema "github.com/DaiYuANg/gity/internal/infrastructure/persistence/db_schema"
	collectionx "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

const (
	StatusOnline  = ciports.ProjectRunnerStatusOnline
	StatusOffline = ciports.ProjectRunnerStatusOffline
)

type Repository struct {
	base *dbxrepo.Base[cidomain.ProjectRunner, dbschema.ProjectRunnerSchemaDef]
}

type CreateInput = ciports.CreateProjectRunnerInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[cidomain.ProjectRunner](db, dbschema.ProjectRunnerSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectRunnerRepository(repo *Repository) ciports.ProjectRunnerRepository {
	return repo
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[cidomain.ProjectRunner], error) {
	query := querydsl.Select(dbschema.ProjectRunnerSchema.AllColumns().Values()...).
		From(dbschema.ProjectRunnerSchema).
		Where(dbschema.ProjectRunnerSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectRunnerSchema.ID.Desc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByProjectAndID(ctx context.Context, projectID, id int64) (cidomain.ProjectRunner, error) {
	query := querydsl.Select(dbschema.ProjectRunnerSchema.AllColumns().Values()...).
		From(dbschema.ProjectRunnerSchema).
		Where(querydsl.And(
			dbschema.ProjectRunnerSchema.ProjectID.Eq(projectID),
			dbschema.ProjectRunnerSchema.ID.Eq(id),
		)).
		Limit(1)
	return persistence.One(r.base.First(ctx, query))
}

func (r *Repository) GetByToken(ctx context.Context, token string) (cidomain.ProjectRunner, error) {
	return persistence.One(r.base.GetByKey(ctx, dbxrepo.Key{
		"token": strings.TrimSpace(token),
	}))
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
	if _, err := dbxrepo.By(r.base, dbschema.ProjectRunnerSchema.ID).Update(ctx, id,
		dbschema.ProjectRunnerSchema.Status.Set(StatusOnline),
		dbschema.ProjectRunnerSchema.LastContactAt.Set(now),
		dbschema.ProjectRunnerSchema.UpdatedAt.Set(now),
	); err != nil {
		return fmt.Errorf("record project runner heartbeat: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := dbxrepo.By(r.base, dbschema.ProjectRunnerSchema.ID).Delete(ctx, id); err != nil {
		return fmt.Errorf("delete project runner: %w", err)
	}
	return nil
}

func normalizeTags(value string) string {
	parts := strings.Split(value, ",")
	seen := setx.NewSetWithCapacity[string](len(parts))
	out := collectionx.NewListWithCapacity[string](len(parts))
	for _, part := range parts {
		trimmed := strings.TrimSpace(strings.ToLower(part))
		if trimmed == "" {
			continue
		}
		if seen.Contains(trimmed) {
			continue
		}
		seen.Add(trimmed)
		out.Add(trimmed)
	}
	return strings.Join(out.Values(), ",")
}
