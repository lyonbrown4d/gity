package ports

import (
	"context"

	collectionx "github.com/arcgolabs/collectionx/list"
	auditdomain "github.com/lyonbrown4d/gity/internal/domain/audit"
)

type ProjectAuditEventRepository interface {
	Create(ctx context.Context, input CreateProjectAuditEventInput) (auditdomain.ProjectAuditEvent, error)
	ListByProjectID(ctx context.Context, projectID int64, limit int) (*collectionx.List[auditdomain.ProjectAuditEvent], error)
}

type CreateProjectAuditEventInput struct {
	ProjectID      int64
	OrganizationID int64
	EventName      string
	Action         string
	ActorUserID    int64
	TargetType     string
	TargetID       string
	Summary        string
	Payload        string
}
