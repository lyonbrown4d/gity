package issue

import (
	"strconv"

	issuedomain "github.com/DaiYuANg/gity/internal/domain/issue"
)

func toIssueView(item issuedomain.ProjectIssue) issueView {
	return issueView{
		ID:           strconv.FormatInt(item.IID, 10),
		RepositoryID: strconv.FormatInt(item.ProjectID, 10),
		Number:       item.IID,
		Title:        item.Title,
		Description:  item.Description,
		Status:       stateToStatus(item.State),
		AuthorUserID: strconv.FormatInt(item.AuthorUserID, 10),
		CreatedAt:    item.CreatedAt.UTC().Format("2006-01-02T15:04:05Z"),
		UpdatedAt:    item.UpdatedAt.UTC().Format("2006-01-02T15:04:05Z"),
	}
}

func toIssueCommentView(issueIID int64, item issuedomain.ProjectIssueComment) issueCommentView {
	return issueCommentView{
		ID:           strconv.FormatInt(item.ID, 10),
		IssueID:      strconv.FormatInt(issueIID, 10),
		AuthorUserID: strconv.FormatInt(item.AuthorUserID, 10),
		Content:      item.Body,
		CreatedAt:    item.CreatedAt.UTC().Format("2006-01-02T15:04:05Z"),
		UpdatedAt:    item.UpdatedAt.UTC().Format("2006-01-02T15:04:05Z"),
	}
}

func stateToStatus(state string) string {
	if state == "closed" {
		return "closed"
	}
	return "open"
}

func statusToState(status string) string {
	if status == "open" {
		return "opened"
	}
	return status
}

func (in createIssueInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in updateIssueInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createCommentInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createAttachmentInput) AuthorizationHeader() string {
	return in.Authorization
}
