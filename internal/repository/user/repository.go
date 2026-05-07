package user

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
	base *dbxrepo.Base[entity.User, entity.UserSchemaDef]
}

type CreateInput struct {
	Username    string
	DisplayName string
	Email       string
}

type UpdateInput struct {
	Username    *string
	DisplayName *string
	Email       *string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[entity.User](db, entity.UserSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) List(ctx context.Context) (*collectionx.List[entity.User], error) {
	query := dbx.Select(entity.UserSchema.AllColumns().Values()...).
		From(entity.UserSchema).
		OrderBy(entity.UserSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByID(ctx context.Context, id int64) (entity.User, error) {
	return r.base.GetByID(ctx, id)
}

func (r *Repository) GetByUsername(ctx context.Context, username string) (entity.User, error) {
	return r.base.GetByKey(ctx, dbxrepo.Key{
		"username": strings.TrimSpace(username),
	})
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

func (r *Repository) UpdateByID(ctx context.Context, id int64, input UpdateInput) error {
	assignments := []dbx.Assignment{}
	if input.Username != nil {
		assignments = append(assignments, entity.UserSchema.Username.Set(strings.TrimSpace(*input.Username)))
	}
	if input.DisplayName != nil {
		assignments = append(assignments, entity.UserSchema.DisplayName.Set(strings.TrimSpace(*input.DisplayName)))
	}
	if input.Email != nil {
		assignments = append(assignments, entity.UserSchema.Email.Set(strings.TrimSpace(*input.Email)))
	}
	assignments = append(assignments, entity.UserSchema.UpdatedAt.Set(time.Now().UTC()))
	if _, err := r.base.UpdateByID(ctx, id, assignments...); err != nil {
		return fmt.Errorf("update user: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := r.base.DeleteByID(ctx, id); err != nil {
		return fmt.Errorf("delete user: %w", err)
	}
	return nil
}
