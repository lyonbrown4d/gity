package ports

import (
	"context"

	namespacedomain "github.com/DaiYuANg/gity/internal/domain/namespace"
	collectionx "github.com/arcgolabs/collectionx/list"
)

type NamespaceRepository interface {
	List(ctx context.Context) (*collectionx.List[namespacedomain.Namespace], error)
	GetByID(ctx context.Context, id int64) (namespacedomain.Namespace, error)
	Create(ctx context.Context, input CreateNamespaceInput) (namespacedomain.Namespace, error)
	UpdateByID(ctx context.Context, id int64, input UpdateNamespaceInput) error
	DeleteByID(ctx context.Context, id int64) error
}

type NamespaceMemberRepository interface {
	ListByNamespaceID(ctx context.Context, namespaceID int64) (*collectionx.List[namespacedomain.NamespaceMember], error)
	FindByNamespaceAndUser(ctx context.Context, namespaceID, userID int64) (namespacedomain.NamespaceMember, error)
	Create(ctx context.Context, input CreateNamespaceMemberInput) (namespacedomain.NamespaceMember, error)
}

type CreateNamespaceInput struct {
	Kind        string
	Name        string
	PathKey     string
	Description string
}

type UpdateNamespaceInput struct {
	Kind        *string
	Name        *string
	PathKey     *string
	Description *string
}

type CreateNamespaceMemberInput struct {
	NamespaceID int64
	UserID      int64
	Role        string
}
