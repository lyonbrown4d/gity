package merge

import domainevent "github.com/lyonbrown4d/gity/internal/domain/event"

const EventProjectMergeRequestMerged = "project.merge_request.merged"

type ProjectMergeRequestMerged struct {
	domainevent.Metadata
	ProjectID      int64  `json:"project_id"`
	MergeRequestID int64  `json:"merge_request_id"`
	MergeIID       int64  `json:"merge_iid"`
	ActorUserID    int64  `json:"actor_user_id"`
	AuthorUserID   int64  `json:"author_user_id"`
	Title          string `json:"title"`
	SourceBranch   string `json:"source_branch"`
	TargetBranch   string `json:"target_branch"`
	State          string `json:"state"`
}

func (ProjectMergeRequestMerged) Name() string {
	return EventProjectMergeRequestMerged
}

func NewProjectMergeRequestMergedEvent(mr ProjectMergeRequest, actorUserID int64) ProjectMergeRequestMerged {
	return ProjectMergeRequestMerged{
		Metadata:       domainevent.NewMetadata(),
		ProjectID:      mr.ProjectID,
		MergeRequestID: mr.ID,
		MergeIID:       mr.IID,
		ActorUserID:    actorUserID,
		AuthorUserID:   mr.AuthorUserID,
		Title:          mr.Title,
		SourceBranch:   mr.SourceBranch,
		TargetBranch:   mr.TargetBranch,
		State:          mr.State,
	}
}
