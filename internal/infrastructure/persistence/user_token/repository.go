package usertoken

import (
	"context"
	"fmt"
	identityports "github.com/DaiYuANg/gity/internal/application/ports"
	identity "github.com/DaiYuANg/gity/internal/domain/identity"
	persistence "github.com/DaiYuANg/gity/internal/infrastructure/persistence"
	dbschema "github.com/DaiYuANg/gity/internal/infrastructure/persistence/db_schema"
	"strings"
	"time"

	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
)

type Repository struct {
	base *dbxrepo.Base[identity.UserAccessToken, dbschema.UserAccessTokenSchemaDef]
}

type CreateInput = identityports.CreateUserAccessTokenInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[identity.UserAccessToken](db, dbschema.UserAccessTokenSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func NewUserAccessTokenRepository(repo *Repository) identityports.UserAccessTokenRepository {
	return repo
}

func (r *Repository) ListByUserID(ctx context.Context, userID int64) (*collectionx.List[identity.UserAccessToken], error) {
	query := querydsl.Select(dbschema.UserAccessTokenSchema.AllColumns().Values()...).
		From(dbschema.UserAccessTokenSchema).
		Where(dbschema.UserAccessTokenSchema.UserID.Eq(userID)).
		OrderBy(dbschema.UserAccessTokenSchema.ID.Desc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByToken(ctx context.Context, token string) (identity.UserAccessToken, error) {
	return persistence.One(r.base.GetByKey(ctx, dbxrepo.Key{
		"token": strings.TrimSpace(token),
	}))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (identity.UserAccessToken, error) {
	now := time.Now().UTC()
	item := identity.UserAccessToken{
		UserID:    input.UserID,
		Name:      strings.TrimSpace(input.Name),
		Token:     strings.TrimSpace(input.Token),
		CreatedAt: now,
		UpdatedAt: now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return identity.UserAccessToken{}, fmt.Errorf("insert user token: %w", err)
	}
	return item, nil
}

func (r *Repository) DeleteByToken(ctx context.Context, token string) error {
	record, err := r.GetByToken(ctx, token)
	if err != nil {
		return err
	}
	if _, err := r.base.DeleteByID(ctx, record.ID); err != nil {
		return fmt.Errorf("delete user token: %w", err)
	}
	return nil
}
