package usertoken

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

type Repository struct {
	base *dbxrepo.Base[entity.UserAccessToken, entity.UserAccessTokenSchemaDef]
}

type CreateInput struct {
	UserID int64
	Name   string
	Token  string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[entity.UserAccessToken](db, entity.UserAccessTokenSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByUserID(ctx context.Context, userID int64) (*collectionx.List[entity.UserAccessToken], error) {
	query := dbx.Select(entity.UserAccessTokenSchema.AllColumns().Values()...).
		From(entity.UserAccessTokenSchema).
		Where(entity.UserAccessTokenSchema.UserID.Eq(userID)).
		OrderBy(entity.UserAccessTokenSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByToken(ctx context.Context, token string) (entity.UserAccessToken, error) {
	return r.base.GetByKey(ctx, dbxrepo.Key{
		"token": strings.TrimSpace(token),
	})
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.UserAccessToken, error) {
	now := time.Now().UTC()
	item := entity.UserAccessToken{
		UserID:    input.UserID,
		Name:      strings.TrimSpace(input.Name),
		Token:     strings.TrimSpace(input.Token),
		CreatedAt: now,
		UpdatedAt: now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return entity.UserAccessToken{}, fmt.Errorf("insert user token: %w", err)
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
