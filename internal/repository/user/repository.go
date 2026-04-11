package user

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/DaiYuANg/arcgo/collectionx"
	"github.com/DaiYuANg/arcgo/dbx"
	dbxrepo "github.com/DaiYuANg/arcgo/dbx/repository"
	"github.com/DaiYuANg/gity/internal/entity"
)

type Repository struct {
	base *dbxrepo.Base[entity.User, entity.UserSchemaDef]
}

type CreateInput struct {
	Username    string
	DisplayName string
	Email       string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[entity.User](db, entity.UserSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) List(ctx context.Context) (collectionx.List[entity.User], error) {
	query := dbx.Select(entity.UserSchema.AllColumns().Values()...).
		From(entity.UserSchema).
		OrderBy(entity.UserSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByID(ctx context.Context, id int64) (entity.User, error) {
	return r.base.GetByID(ctx, id)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.User, error) {
	now := time.Now().UTC()
	item := entity.User{
		Username:    strings.TrimSpace(input.Username),
		DisplayName: strings.TrimSpace(input.DisplayName),
		Email:       strings.TrimSpace(input.Email),
		CreatedAt:   now,
		UpdatedAt:   now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return entity.User{}, fmt.Errorf("insert user: %w", err)
	}
	return item, nil
}
