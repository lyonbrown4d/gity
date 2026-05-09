package audit

import "context"

func (e *Endpoint) listProjectAuditEvents(ctx context.Context, in *projectAuditEventsInput) (*auditOutput, error) {
	items, err := e.service.ListProjectEvents(ctx, in.ProjectID, in.Limit)
	if err != nil {
		return nil, err
	}
	return &auditOutput{Body: items}, nil
}
