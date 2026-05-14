package merge

import "time"

type ProjectMergeRequestApprovalRule struct {
	ID                int64     `dbx:"id"`
	ProjectID         int64     `dbx:"project_id"`
	Name              string    `dbx:"name"`
	TargetBranch      string    `dbx:"target_branch"`
	ApprovalsRequired int       `dbx:"approvals_required"`
	EligibleUserIDs   string    `dbx:"eligible_user_ids"`
	CodeOwner         int       `dbx:"code_owner"`
	CreatedAt         time.Time `dbx:"created_at"`
	UpdatedAt         time.Time `dbx:"updated_at"`
}

func (r ProjectMergeRequestApprovalRule) RequiresCodeOwner() bool {
	return r.CodeOwner != 0
}
