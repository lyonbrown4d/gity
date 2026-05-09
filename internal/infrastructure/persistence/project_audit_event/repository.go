package projectauditevent

import (
	"context"
	"strings"
	"time"

	auditports "github.com/DaiYuANg/gity/internal/application/ports"
	auditdomain "github.com/DaiYuANg/gity/internal/domain/audit"
	persistence "github.com/DaiYuANg/gity/internal/infrastructure/persistence"
	dbschema "github.com/DaiYuANg/gity/internal/infrastructure/persistence/db_schema"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"github.com/samber/oops"
)

type Repository struct {
	base *dbxrepo.Base[auditdomain.ProjectAuditEvent, dbschema.ProjectAuditEventSchemaDef]
}

type CreateInput = auditports.CreateProjectAuditEventInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[auditdomain.ProjectAuditEvent](db, dbschema.ProjectAuditEventSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectAuditEventRepository(repo *Repository) auditports.ProjectAuditEventRepository {
	return repo
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (auditdomain.ProjectAuditEvent, error) {
	now := time.Now().UTC()
	item := auditdomain.ProjectAuditEvent{
		ProjectID:      input.ProjectID,
		OrganizationID: input.OrganizationID,
		EventName:      strings.TrimSpace(input.EventName),
		Action:         strings.TrimSpace(input.Action),
		ActorUserID:    input.ActorUserID,
		TargetType:     strings.TrimSpace(input.TargetType),
		TargetID:       strings.TrimSpace(input.TargetID),
		Summary:        strings.TrimSpace(input.Summary),
		Payload:        input.Payload,
		CreatedAt:      now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return auditdomain.ProjectAuditEvent{}, oops.In("persistence.audit").With("project_id", input.ProjectID, "event_name", input.EventName).Wrapf(err, "insert project audit event")
	}
	return item, nil
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64, limit int) (*collectionx.List[auditdomain.ProjectAuditEvent], error) {
	if limit <= 0 || limit > 200 {
		limit = 50
	}
	query := querydsl.Select(dbschema.ProjectAuditEventSchema.AllColumns().Values()...).
		From(dbschema.ProjectAuditEventSchema).
		Where(dbschema.ProjectAuditEventSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectAuditEventSchema.CreatedAt.Desc()).
		Limit(limit)
	return persistence.Many(r.base.List(ctx, query))
}
