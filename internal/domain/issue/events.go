package issue

import domainevent "github.com/lyonbrown4d/gity/internal/domain/event"

const (
	EventProjectIssueCreated   = "project.issue.created"
	EventProjectIssueAssigned  = "project.issue.assigned"
	EventProjectIssueCommented = "project.issue.commented"
)

type ProjectIssueCreated struct {
	domainevent.Metadata
	ProjectID    int64  `json:"project_id"`
	IssueID      int64  `json:"issue_id"`
	IssueIID     int64  `json:"issue_iid"`
	AuthorUserID int64  `json:"author_user_id"`
	Title        string `json:"title"`
	State        string `json:"state"`
}

func (ProjectIssueCreated) Name() string {
	return EventProjectIssueCreated
}

func NewProjectIssueCreatedEvent(issue ProjectIssue) ProjectIssueCreated {
	return ProjectIssueCreated{
		Metadata:     domainevent.NewMetadata(),
		ProjectID:    issue.ProjectID,
		IssueID:      issue.ID,
		IssueIID:     issue.IID,
		AuthorUserID: issue.AuthorUserID,
		Title:        issue.Title,
		State:        issue.State,
	}
}

type ProjectIssueAssigned struct {
	domainevent.Metadata
	ProjectID      int64  `json:"project_id"`
	IssueID        int64  `json:"issue_id"`
	IssueIID       int64  `json:"issue_iid"`
	AuthorUserID   int64  `json:"author_user_id"`
	AssigneeUserID int64  `json:"assignee_user_id"`
	ActorUserID    int64  `json:"actor_user_id"`
	Title          string `json:"title"`
}

func (ProjectIssueAssigned) Name() string {
	return EventProjectIssueAssigned
}

func NewProjectIssueAssignedEvent(issue ProjectIssue, assigneeUserID, actorUserID int64) ProjectIssueAssigned {
	return ProjectIssueAssigned{
		Metadata:       domainevent.NewMetadata(),
		ProjectID:      issue.ProjectID,
		IssueID:        issue.ID,
		IssueIID:       issue.IID,
		AuthorUserID:   issue.AuthorUserID,
		AssigneeUserID: assigneeUserID,
		ActorUserID:    actorUserID,
		Title:          issue.Title,
	}
}

type ProjectIssueCommented struct {
	domainevent.Metadata
	ProjectID           int64  `json:"project_id"`
	IssueID             int64  `json:"issue_id"`
	IssueIID            int64  `json:"issue_iid"`
	CommentID           int64  `json:"comment_id"`
	AuthorUserID        int64  `json:"author_user_id"`
	CommentAuthorUserID int64  `json:"comment_author_user_id"`
	Title               string `json:"title"`
}

func (ProjectIssueCommented) Name() string {
	return EventProjectIssueCommented
}

func NewProjectIssueCommentedEvent(issue ProjectIssue, comment ProjectIssueComment) ProjectIssueCommented {
	return ProjectIssueCommented{
		Metadata:            domainevent.NewMetadata(),
		ProjectID:           issue.ProjectID,
		IssueID:             issue.ID,
		IssueIID:            issue.IID,
		CommentID:           comment.ID,
		AuthorUserID:        issue.AuthorUserID,
		CommentAuthorUserID: comment.AuthorUserID,
		Title:               issue.Title,
	}
}
