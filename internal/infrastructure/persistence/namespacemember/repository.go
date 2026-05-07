package namespacemember

import (
	"context"
	"fmt"
	namespacedomain "github.com/DaiYuANg/gity/internal/domain/namespace"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[namespacedomain.NamespaceMember, namespacedomain.NamespaceMemberSchemaDef]
}

type CreateInput struct {
	NamespaceID int64
	UserID      int64
	Role        string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[namespacedomain.NamespaceMember](db, namespacedomain.NamespaceMemberSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByNamespaceID(ctx context.Context, namespaceID int64) (*collectionx.List[namespacedomain.NamespaceMember], error) {
	query := querydsl.Select(namespacedomain.NamespaceMemberSchema.AllColumns().Values()...).
		From(namespacedomain.NamespaceMemberSchema).
		Where(namespacedomain.NamespaceMemberSchema.NamespaceID.Eq(namespaceID)).
		OrderBy(namespacedomain.NamespaceMemberSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) FindByNamespaceAndUser(ctx context.Context, namespaceID int64, userID int64) (namespacedomain.NamespaceMember, error) {
	return r.base.GetByKey(ctx, dbxrepo.Key{
		"namespace_id": namespaceID,
		"user_id":      userID,
	})
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (namespacedomain.NamespaceMember, error) {
	now := time.Now().UTC()
	item := namespacedomain.NamespaceMember{
		NamespaceID: input.NamespaceID,
		UserID:      input.UserID,
		Role:        input.Role,
		CreatedAt:   now,
		UpdatedAt:   now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return namespacedomain.NamespaceMember{}, fmt.Errorf("insert namespace member: %w", err)
	}
	return item, nil
}
