package namespacemember

import (
	"context"
	"fmt"
	namespaceports "github.com/DaiYuANg/gity/internal/application/ports"
	namespacedomain "github.com/DaiYuANg/gity/internal/domain/namespace"
	persistence "github.com/DaiYuANg/gity/internal/infrastructure/persistence"
	dbschema "github.com/DaiYuANg/gity/internal/infrastructure/persistence/db_schema"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[namespacedomain.NamespaceMember, dbschema.NamespaceMemberSchemaDef]
}

type CreateInput = namespaceports.CreateNamespaceMemberInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[namespacedomain.NamespaceMember](db, dbschema.NamespaceMemberSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewNamespaceMemberRepository(repo *Repository) namespaceports.NamespaceMemberRepository {
	return repo
}

func (r *Repository) ListByNamespaceID(ctx context.Context, namespaceID int64) (*collectionx.List[namespacedomain.NamespaceMember], error) {
	query := querydsl.Select(dbschema.NamespaceMemberSchema.AllColumns().Values()...).
		From(dbschema.NamespaceMemberSchema).
		Where(dbschema.NamespaceMemberSchema.NamespaceID.Eq(namespaceID)).
		OrderBy(dbschema.NamespaceMemberSchema.ID.Desc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) FindByNamespaceAndUser(ctx context.Context, namespaceID int64, userID int64) (namespacedomain.NamespaceMember, error) {
	return persistence.One(r.base.GetByKey(ctx, dbxrepo.Key{
		"namespace_id": namespaceID,
		"user_id":      userID,
	}))
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
