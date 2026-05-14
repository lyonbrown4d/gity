package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	organizationdomain "github.com/lyonbrown4d/gity/internal/domain/organization"
)

type OrganizationMemberSchemaDef struct {
	schema.Schema[organizationdomain.OrganizationMember]
	ID             column.IDColumn[organizationdomain.OrganizationMember, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	OrganizationID column.Column[organizationdomain.OrganizationMember, int64]                      `dbx:"organization_id,index,ref=organizations.id,ondelete=cascade"`
	UserID         column.Column[organizationdomain.OrganizationMember, int64]                      `dbx:"user_id,index,ref=users.id,ondelete=cascade"`
	Role           column.Column[organizationdomain.OrganizationMember, string]                     `dbx:"role,index"`
	CreatedAt      column.Column[organizationdomain.OrganizationMember, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt      column.Column[organizationdomain.OrganizationMember, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var OrganizationMemberSchema = schema.MustSchema("organization_members", OrganizationMemberSchemaDef{})
