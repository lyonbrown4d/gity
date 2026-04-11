package namespacemember

import (
	"context"
	"fmt"
	"time"

	"github.com/DaiYuANg/arcgo/collectionx"
	"github.com/DaiYuANg/arcgo/dbx"
	dbxrepo "github.com/DaiYuANg/arcgo/dbx/repository"
	"github.com/DaiYuANg/gity/internal/entity"
)

type Repository struct {
	base *dbxrepo.Base[entity.NamespaceMember, entity.NamespaceMemberSchemaDef]
}

type CreateInput struct {
	NamespaceID int64
	UserID      int64
	Role        string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[entity.NamespaceMember](db, entity.NamespaceMemberSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByNamespaceID(ctx context.Context, namespaceID int64) (collectionx.List[entity.NamespaceMember], error) {
	query := dbx.Select(entity.NamespaceMemberSchema.AllColumns().Values()...).
		From(entity.NamespaceMemberSchema).
		Where(entity.NamespaceMemberSchema.NamespaceID.Eq(namespaceID)).
		OrderBy(entity.NamespaceMemberSchema.ID.Desc())
	return r.base.List(ctx, query)
}

func (r *Repository) FindByNamespaceAndUser(ctx context.Context, namespaceID int64, userID int64) (entity.NamespaceMember, error) {
	return r.base.GetByKey(ctx, dbxrepo.Key{
		"namespace_id": namespaceID,
		"user_id":      userID,
	})
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.NamespaceMember, error) {
	now := time.Now().UTC()
	item := entity.NamespaceMember{
		NamespaceID: input.NamespaceID,
		UserID:      input.UserID,
		Role:        input.Role,
		CreatedAt:   now,
		UpdatedAt:   now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return entity.NamespaceMember{}, fmt.Errorf("insert namespace member: %w", err)
	}
	return item, nil
}
