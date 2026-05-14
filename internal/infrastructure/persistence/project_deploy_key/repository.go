package projectdeploykey

import (
	"context"
	"fmt"
	"strings"
	"time"

	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	identityports "github.com/lyonbrown4d/gity/internal/application/ports"
	identity "github.com/lyonbrown4d/gity/internal/domain/identity"
	persistence "github.com/lyonbrown4d/gity/internal/infrastructure/persistence"
	dbaudit "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_audit"
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
)

type Repository struct {
	base *dbxrepo.Base[identity.ProjectDeployKey, dbschema.ProjectDeployKeySchemaDef]
}

type CreateInput = identityports.CreateProjectDeployKeyInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[identity.ProjectDeployKey](
			db,
			dbschema.ProjectDeployKeySchema,
			dbxrepo.WithKeyNotFoundAsError(true),
			dbxrepo.WithAuditWriter(dbaudit.ProjectDeployKeyAudit()),
		),
	}, nil
}

func NewProjectDeployKeyRepository(repo *Repository) identityports.ProjectDeployKeyRepository {
	return repo
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[identity.ProjectDeployKey], error) {
	query := querydsl.Select(dbschema.ProjectDeployKeySchema.AllColumns().Values()...).
		From(dbschema.ProjectDeployKeySchema).
		Where(dbschema.ProjectDeployKeySchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectDeployKeySchema.ID.Desc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByID(ctx context.Context, id int64) (identity.ProjectDeployKey, error) {
	return persistence.One(dbxrepo.By(r.base, dbschema.ProjectDeployKeySchema.ID).Get(ctx, id))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (identity.ProjectDeployKey, error) {
	now := time.Now().UTC()
	item := identity.ProjectDeployKey{
		ProjectID:       input.ProjectID,
		Title:           strings.TrimSpace(input.Title),
		Fingerprint:     strings.TrimSpace(input.Fingerprint),
		PublicKey:       strings.TrimSpace(input.PublicKey),
		CanPush:         boolInt(input.CanPush),
		CreatedByUserID: input.CreatedByUserID,
		CreatedAt:       now,
		UpdatedAt:       now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return identity.ProjectDeployKey{}, fmt.Errorf("insert project deploy key: %w", err)
	}
	return item, nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := dbxrepo.By(r.base, dbschema.ProjectDeployKeySchema.ID).Delete(ctx, id); err != nil {
		return fmt.Errorf("delete project deploy key: %w", err)
	}
	return nil
}

func boolInt(value bool) int {
	if value {
		return 1
	}
	return 0
}
