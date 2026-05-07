package entity

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
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
	schema.Schema[ProjectMergeRequest]
	ID           column.IDColumn[ProjectMergeRequest, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	ProjectID    column.Column[ProjectMergeRequest, int64]                      `dbx:"project_id,index,ref=projects.id,ondelete=cascade"`
	IID          column.Column[ProjectMergeRequest, int64]                      `dbx:"iid,index"`
	AuthorUserID column.Column[ProjectMergeRequest, int64]                      `dbx:"author_user_id,index,ref=users.id,ondelete=restrict"`
	Title        column.Column[ProjectMergeRequest, string]                     `dbx:"title"`
	Description  column.Column[ProjectMergeRequest, string]                     `dbx:"description,null"`
	State        column.Column[ProjectMergeRequest, string]                     `dbx:"state,index"`
	SourceBranch column.Column[ProjectMergeRequest, string]                     `dbx:"source_branch,index"`
	TargetBranch column.Column[ProjectMergeRequest, string]                     `dbx:"target_branch,index"`
	CreatedAt    column.Column[ProjectMergeRequest, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt    column.Column[ProjectMergeRequest, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`
}

var ProjectMergeRequestSchema = schema.MustSchema("project_merge_requests", ProjectMergeRequestSchemaDef{})
