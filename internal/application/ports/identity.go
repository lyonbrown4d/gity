package ports

import (
	"context"
	"time"

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

type ProjectAccessTokenRepository interface {
	ListByProjectIDAndKind(ctx context.Context, projectID int64, kind string) (*collectionx.List[identitydomain.ProjectAccessToken], error)
	GetByID(ctx context.Context, id int64) (identitydomain.ProjectAccessToken, error)
	GetByToken(ctx context.Context, token string) (identitydomain.ProjectAccessToken, error)
	Create(ctx context.Context, input CreateProjectAccessTokenInput) (identitydomain.ProjectAccessToken, error)
	RevokeByID(ctx context.Context, id int64) error
}

type ProjectDeployKeyRepository interface {
	ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[identitydomain.ProjectDeployKey], error)
	GetByID(ctx context.Context, id int64) (identitydomain.ProjectDeployKey, error)
	Create(ctx context.Context, input CreateProjectDeployKeyInput) (identitydomain.ProjectDeployKey, error)
	DeleteByID(ctx context.Context, id int64) error
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

type CreateProjectAccessTokenInput struct {
	ProjectID       int64
	Kind            string
	Name            string
	Username        string
	Token           string
	Scopes          string
	CreatedByUserID int64
	ExpiresAt       time.Time
}

type CreateProjectDeployKeyInput struct {
	ProjectID       int64
	Title           string
	Fingerprint     string
	PublicKey       string
	CanPush         bool
	CreatedByUserID int64
}
