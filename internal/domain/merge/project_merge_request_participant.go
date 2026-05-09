package merge

import (
	"strings"
	"time"
)

const (
	ProjectMergeRequestParticipantRoleReviewer = "reviewer"
	ProjectMergeRequestParticipantRoleAssignee = "assignee"
)

type ProjectMergeRequestParticipant struct {
	ID             int64     `dbx:"id"`
	MergeRequestID int64     `dbx:"merge_request_id"`
	UserID         int64     `dbx:"user_id"`
	Role           string    `dbx:"role"`
	CreatedAt      time.Time `dbx:"created_at"`
	UpdatedAt      time.Time `dbx:"updated_at"`
}

func NormalizeProjectMergeRequestParticipantRole(value string) string {
	switch strings.ToLower(strings.TrimSpace(value)) {
	case ProjectMergeRequestParticipantRoleReviewer:
		return ProjectMergeRequestParticipantRoleReviewer
	case ProjectMergeRequestParticipantRoleAssignee:
		return ProjectMergeRequestParticipantRoleAssignee
	default:
		return ""
	}
}
