package ports

import (
	"context"

	organizationdomain "github.com/DaiYuANg/gity/internal/domain/organization"
	collectionx "github.com/arcgolabs/collectionx/list"
)

type OrganizationRepository interface {
	List(ctx context.Context) (*collectionx.List[organizationdomain.Organization], error)
	GetByID(ctx context.Context, id int64) (organizationdomain.Organization, error)
	Create(ctx context.Context, input CreateOrganizationInput) (organizationdomain.Organization, error)
	UpdateByID(ctx context.Context, id int64, input UpdateOrganizationInput) error
	DeleteByID(ctx context.Context, id int64) error
}

type OrganizationMemberRepository interface {
	ListByOrganizationID(ctx context.Context, organizationID int64) (*collectionx.List[organizationdomain.OrganizationMember], error)
	FindByOrganizationAndUser(ctx context.Context, organizationID, userID int64) (organizationdomain.OrganizationMember, error)
	Create(ctx context.Context, input CreateOrganizationMemberInput) (organizationdomain.OrganizationMember, error)
}

type CreateOrganizationInput struct {
	Name        string
	PathKey     string
	Description string
	Visibility  string
}

type UpdateOrganizationInput struct {
	Name        *string
	PathKey     *string
	Description *string
	Visibility  *string
}

type CreateOrganizationMemberInput struct {
	OrganizationID int64
	UserID         int64
	Role           string
}
