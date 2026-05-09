package dbschema

import (
	"time"

	auditdomain "github.com/DaiYuANg/gity/internal/domain/audit"
	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
)

type ProjectAuditEventSchemaDef struct {
	schema.Schema[auditdomain.ProjectAuditEvent]
	ID             column.IDColumn[auditdomain.ProjectAuditEvent, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID      column.Column[auditdomain.ProjectAuditEvent, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	OrganizationID column.Column[auditdomain.ProjectAuditEvent, int64]                      `dbx:"organization_id,index,ref=organizations.id,ondelete=cascade"`
	EventName      column.Column[auditdomain.ProjectAuditEvent, string]                     `dbx:"event_name,index"`
	Action         column.Column[auditdomain.ProjectAuditEvent, string]                     `dbx:"action,index"`
	ActorUserID    column.Column[auditdomain.ProjectAuditEvent, int64]                      `dbx:"actor_user_id,index"`
	TargetType     column.Column[auditdomain.ProjectAuditEvent, string]                     `dbx:"target_type,index"`
	TargetID       column.Column[auditdomain.ProjectAuditEvent, string]                     `dbx:"target_id,index"`
	Summary        column.Column[auditdomain.ProjectAuditEvent, string]                     `dbx:"summary"`
	Payload        column.Column[auditdomain.ProjectAuditEvent, string]                     `dbx:"payload,type=TEXT"`
	CreatedAt      column.Column[auditdomain.ProjectAuditEvent, time.Time]                  `dbx:"created_at,type=TIMESTAMP,index"`
}

var ProjectAuditEventSchema = schema.MustSchema("project_audit_events", ProjectAuditEventSchemaDef{})
