// Package audit defines audit log domain models.
package audit

import "time"

type ProjectAuditEvent struct {
	ID             int64     `dbx:"id"`
	ProjectID      int64     `dbx:"project_id"`
	OrganizationID int64     `dbx:"organization_id"`
	EventName      string    `dbx:"event_name"`
	Action         string    `dbx:"action"`
	ActorUserID    int64     `dbx:"actor_user_id"`
	TargetType     string    `dbx:"target_type"`
	TargetID       string    `dbx:"target_id"`
	Summary        string    `dbx:"summary"`
	Payload        string    `dbx:"payload"`
	CreatedAt      time.Time `dbx:"created_at"`
}
