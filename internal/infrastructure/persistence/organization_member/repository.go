package organizationmember

import (
	"context"
	"fmt"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	organizationports "github.com/lyonbrown4d/gity/internal/application/ports"
	organizationdomain "github.com/lyonbrown4d/gity/internal/domain/organization"
	persistence "github.com/lyonbrown4d/gity/internal/infrastructure/persistence"
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[organizationdomain.OrganizationMember, dbschema.OrganizationMemberSchemaDef]
}

type CreateInput = organizationports.CreateOrganizationMemberInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[organizationdomain.OrganizationMember](db, dbschema.OrganizationMemberSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewOrganizationMemberRepository(repo *Repository) organizationports.OrganizationMemberRepository {
	return repo
}

func (r *Repository) ListByOrganizationID(ctx context.Context, organizationID int64) (*collectionx.List[organizationdomain.OrganizationMember], error) {
	query := querydsl.Select(dbschema.OrganizationMemberSchema.AllColumns().Values()...).
		From(dbschema.OrganizationMemberSchema).
		Where(dbschema.OrganizationMemberSchema.OrganizationID.Eq(organizationID)).
		OrderBy(dbschema.OrganizationMemberSchema.ID.Desc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) FindByOrganizationAndUser(ctx context.Context, organizationID, userID int64) (organizationdomain.OrganizationMember, error) {
	return persistence.One(r.base.GetByKey(ctx, dbxrepo.Key{
		"organization_id": organizationID,
		"user_id":         userID,
	}))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (organizationdomain.OrganizationMember, error) {
	now := time.Now().UTC()
	item := organizationdomain.OrganizationMember{
		OrganizationID: input.OrganizationID,
		UserID:         input.UserID,
		Role:           input.Role,
		CreatedAt:      now,
		UpdatedAt:      now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return organizationdomain.OrganizationMember{}, fmt.Errorf("insert organization member: %w", err)
	}
	return item, nil
}
