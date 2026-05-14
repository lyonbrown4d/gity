package ports

import (
	"context"

	collectionx "github.com/arcgolabs/collectionx/list"
	identitydomain "github.com/lyonbrown4d/gity/internal/domain/identity"
)

type UserRepository interface {
	List(ctx context.Context) (*collectionx.List[identitydomain.User], error)
	GetByID(ctx context.Context, id int64) (identitydomain.User, error)
	GetByUsername(ctx context.Context, username string) (identitydomain.User, error)
	Create(ctx context.Context, input CreateUserInput) (identitydomain.User, error)
	UpdateByID(ctx context.Context, id int64, input UpdateUserInput) error
	DeleteByID(ctx context.Context, id int64) error
}

type UserAccessTokenRepository interface {
	ListByUserID(ctx context.Context, userID int64) (*collectionx.List[identitydomain.UserAccessToken], error)
	GetByToken(ctx context.Context, token string) (identitydomain.UserAccessToken, error)
	Create(ctx context.Context, input CreateUserAccessTokenInput) (identitydomain.UserAccessToken, error)
	DeleteByToken(ctx context.Context, token string) error
}

type CreateUserInput struct {
	Username     string
	DisplayName  string
	Email        string
	IsSuperAdmin bool
}

type UpdateUserInput struct {
	Username     *string
	DisplayName  *string
	Email        *string
	IsSuperAdmin *bool
}

type CreateUserAccessTokenInput struct {
	UserID int64
	Name   string
	Token  string
}
