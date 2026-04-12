package entity

import (
	"time"

	"github.com/DaiYuANg/arcgo/dbx"
)

type ProjectMergeRequest struct {
	ID           int64     `dbx:"id"`
	ProjectID    int64     `dbx:"project_id"`
	IID          int64     `dbx:"iid"`
	AuthorUserID int64     `dbx:"author_user_id"`
	Title        string    `dbx:"title"`
	Description  string    `dbx:"description"`
	State        string    `dbx:"state"`
	SourceBranch string    `dbx:"source_branch"`
	TargetBranch string    `dbx:"target_branch"`
	CreatedAt    time.Time `dbx:"created_at"`
	UpdatedAt    time.Time `dbx:"updated_at"`
}

type ProjectMergeRequestSchemaDef struct {
	dbx.Schema[ProjectMergeRequest]
	ID           dbx.IDColumn[ProjectMergeRequest, int64, dbx.IDSnowflake] `dbx:"id,pk"`
	ProjectID    dbx.Column[ProjectMergeRequest, int64]                    `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	IID          dbx.Column[ProjectMergeRequest, int64]                    `dbx:"iid,index"`
	AuthorUserID dbx.Column[ProjectMergeRequest, int64]                    `dbx:"author_user_id,index,ref=users.id,ondelete=restrict"`
	Title        dbx.Column[ProjectMergeRequest, string]                   `dbx:"title"`
	Description  dbx.Column[ProjectMergeRequest, string]                   `dbx:"description,null"`
	State        dbx.Column[ProjectMergeRequest, string]                   `dbx:"state,index"`
	SourceBranch dbx.Column[ProjectMergeRequest, string]                   `dbx:"source_branch,index"`
	TargetBranch dbx.Column[ProjectMergeRequest, string]                   `dbx:"target_branch,index"`
	CreatedAt    dbx.Column[ProjectMergeRequest, time.Time]                `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt    dbx.Column[ProjectMergeRequest, time.Time]                `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectMergeRequestSchema = dbx.MustSchema("project_merge_requests", ProjectMergeRequestSchemaDef{})
