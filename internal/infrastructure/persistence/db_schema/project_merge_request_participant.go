package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/idgen"
	"github.com/arcgolabs/dbx/schema"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
)

type ProjectMergeRequestParticipantSchemaDef struct {
	schema.Schema[mergedomain.ProjectMergeRequestParticipant]
	ID             column.IDColumn[mergedomain.ProjectMergeRequestParticipant, int64, idgen.IDSnowflake] `dbx:"id,pk"`
	MergeRequestID column.Column[mergedomain.ProjectMergeRequestParticipant, int64]                      `dbx:"merge_request_id,index,ref=project_merge_requests.id,ondelete=cascade"`
	UserID         column.Column[mergedomain.ProjectMergeRequestParticipant, int64]                      `dbx:"user_id,index,ref=users.id,ondelete=cascade"`
	Role           column.Column[mergedomain.ProjectMergeRequestParticipant, string]                     `dbx:"role,index"`
	CreatedAt      column.Column[mergedomain.ProjectMergeRequestParticipant, time.Time]                  `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt      column.Column[mergedomain.ProjectMergeRequestParticipant, time.Time]                  `dbx:"updated_at,type=TIMESTAMP"`

	UniqueMergeRequestUserRole schema.Unique[mergedomain.ProjectMergeRequestParticipant] `idx:"columns=merge_request_id|user_id|role"`
}

var ProjectMergeRequestParticipantSchema = schema.MustSchema("project_merge_request_participants", ProjectMergeRequestParticipantSchemaDef{})
