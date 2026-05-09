package dbschema

import (
	"time"

	organizationdomain "github.com/DaiYuANg/gity/internal/domain/organization"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type OrganizationSchemaDef struct {
	schema.Schema[organizationdomain.Organization]
	ID          column.IDColumn[organizationdomain.Organization, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	Name        column.Column[organizationdomain.Organization, string]                     `dbx:"name"`
	PathKey     column.Column[organizationdomain.Organization, string]                     `dbx:"path_key,unique"`
	FullPath    column.Column[organizationdomain.Organization, string]                     `dbx:"full_path,unique"`
	Description column.Column[organizationdomain.Organization, string]                     `dbx:"description,null"`
	Visibility  column.Column[organizationdomain.Organization, string]                     `dbx:"visibility"`
	CreatedAt   column.Column[organizationdomain.Organization, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt   column.Column[organizationdomain.Organization, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var OrganizationSchema = schema.MustSchema("organizations", OrganizationSchemaDef{})
