// Package todo defines user todo models.
package todo

import "time"

const (
	ProjectTodoStatePending = "pending"
	ProjectTodoStateDone    = "done"

	ProjectTodoKindProjectCreated          = "project_created"
	ProjectTodoKindIssueCreated            = "issue_created"
	ProjectTodoKindIssueAssigned           = "issue_assigned"
	ProjectTodoKindIssueCommented          = "issue_commented"
	ProjectTodoKindBranchProtectionChanged = "branch_protection_changed"
	ProjectTodoKindBranchDeleted           = "branch_deleted"
	ProjectTodoKindMergeRequestMerged      = "merge_request_merged"
)

type ProjectTodo struct {
	ID             int64     `dbx:"id"              json:"id"`
	UserID         int64     `dbx:"user_id"         json:"user_id"`
	OrganizationID int64     `dbx:"organization_id" json:"organization_id"`
	ProjectID      int64     `dbx:"project_id"      json:"project_id"`
	Kind           string    `dbx:"kind"            json:"kind"`
	State          string    `dbx:"state"           json:"state"`
	TargetType     string    `dbx:"target_type"     json:"target_type"`
	TargetID       string    `dbx:"target_id"       json:"target_id"`
	Title          string    `dbx:"title"           json:"title"`
	Summary        string    `dbx:"summary"         json:"summary"`
	ActionURL      string    `dbx:"action_url"      json:"action_url"`
	CreatedAt      time.Time `dbx:"created_at"      json:"created_at"`
	UpdatedAt      time.Time `dbx:"updated_at"      json:"updated_at"`
	DoneAt         time.Time `dbx:"done_at"         json:"done_at,omitzero"`
}
