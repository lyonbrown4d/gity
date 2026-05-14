package projectaccesstoken

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
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
)

type Repository struct {
	base *dbxrepo.Base[identity.ProjectAccessToken, dbschema.ProjectAccessTokenSchemaDef]
}

type CreateInput = identityports.CreateProjectAccessTokenInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[identity.ProjectAccessToken](db, dbschema.ProjectAccessTokenSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectAccessTokenRepository(repo *Repository) identityports.ProjectAccessTokenRepository {
	return repo
}

func (r *Repository) ListByProjectIDAndKind(ctx context.Context, projectID int64, kind string) (*collectionx.List[identity.ProjectAccessToken], error) {
	query := querydsl.Select(dbschema.ProjectAccessTokenSchema.AllColumns().Values()...).
		From(dbschema.ProjectAccessTokenSchema).
		Where(dbschema.ProjectAccessTokenSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectAccessTokenSchema.Kind.Eq(strings.TrimSpace(kind))).
		OrderBy(dbschema.ProjectAccessTokenSchema.ID.Desc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByID(ctx context.Context, id int64) (identity.ProjectAccessToken, error) {
	return persistence.One(dbxrepo.By(r.base, dbschema.ProjectAccessTokenSchema.ID).Get(ctx, id))
}

func (r *Repository) GetByToken(ctx context.Context, token string) (identity.ProjectAccessToken, error) {
	return persistence.One(r.base.GetByKey(ctx, dbxrepo.Key{
		"token": strings.TrimSpace(token),
	}))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (identity.ProjectAccessToken, error) {
	now := time.Now().UTC()
	item := identity.ProjectAccessToken{
		ProjectID:       input.ProjectID,
		Kind:            strings.TrimSpace(input.Kind),
		Name:            strings.TrimSpace(input.Name),
		Username:        strings.TrimSpace(input.Username),
		Token:           strings.TrimSpace(input.Token),
		Scopes:          strings.TrimSpace(input.Scopes),
		CreatedByUserID: input.CreatedByUserID,
		ExpiresAt:       utcOrZero(input.ExpiresAt),
		CreatedAt:       now,
		UpdatedAt:       now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return identity.ProjectAccessToken{}, fmt.Errorf("insert project access token: %w", err)
	}
	return item, nil
}

func (r *Repository) RevokeByID(ctx context.Context, id int64) error {
	now := time.Now().UTC()
	_, err := dbxrepo.PatchSet(r.base, projectAccessTokenKey(id)).
		Set(
			dbschema.ProjectAccessTokenSchema.RevokedAt.Set(now),
			dbschema.ProjectAccessTokenSchema.UpdatedAt.Set(now),
		).
		Apply(ctx)
	if err != nil {
		return fmt.Errorf("revoke project access token: %w", err)
	}
	return nil
}

func projectAccessTokenKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectAccessTokenSchema.ID, id))
}

func utcOrZero(value time.Time) time.Time {
	if value.IsZero() {
		return time.Time{}
	}
	return value.UTC()
}
