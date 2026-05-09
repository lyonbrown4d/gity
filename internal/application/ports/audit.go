package ports

import (
	"context"

	auditdomain "github.com/DaiYuANg/gity/internal/domain/audit"
	collectionx "github.com/arcgolabs/collectionx/list"
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
