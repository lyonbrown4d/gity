package issue

import (
	"strconv"

	issueservice "github.com/lyonbrown4d/gity/internal/application/issue"
	issuedomain "github.com/lyonbrown4d/gity/internal/domain/issue"
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

func toIssueAssigneeView(issueIID int64, item issuedomain.ProjectIssueAssignee) issueAssigneeView {
	return issueAssigneeView{
		ID:        strconv.FormatInt(item.ID, 10),
		IssueID:   strconv.FormatInt(issueIID, 10),
		UserID:    strconv.FormatInt(item.UserID, 10),
		CreatedAt: item.CreatedAt.UTC().Format("2006-01-02T15:04:05Z"),
		UpdatedAt: item.UpdatedAt.UTC().Format("2006-01-02T15:04:05Z"),
	}
}

func toIssueLabelView(issueIID int64, item issuedomain.ProjectIssueLabel) issueLabelView {
	return issueLabelView{
		ID:        strconv.FormatInt(item.ID, 10),
		IssueID:   strconv.FormatInt(issueIID, 10),
		Name:      item.Name,
		Color:     item.Color,
		CreatedAt: item.CreatedAt.UTC().Format("2006-01-02T15:04:05Z"),
		UpdatedAt: item.UpdatedAt.UTC().Format("2006-01-02T15:04:05Z"),
	}
}

func issueLabelsInputFromBody(body issueLabelsBody) []issueservice.LabelInput {
	labels := make([]issueservice.LabelInput, 0, len(body.Labels)+len(body.Names)+len(body.LabelNames))
	for _, label := range body.Labels {
		labels = append(labels, issueservice.LabelInput{Name: label.Name, Color: label.Color})
	}
	for _, name := range body.Names {
		labels = append(labels, issueservice.LabelInput{Name: name})
	}
	for _, name := range body.LabelNames {
		labels = append(labels, issueservice.LabelInput{Name: name})
	}
	return labels
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

func (in createIssueInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in updateIssueInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in updateIssueInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in createCommentInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createCommentInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in setIssueAssigneesInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in setIssueAssigneesInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in setIssueLabelsInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in setIssueLabelsInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in createAttachmentInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in createAttachmentInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in projectIssuesInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectIssuesInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in projectIssueInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectIssueInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in projectAttachmentInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectAttachmentInput) ProjectIDValue() int64 {
	return in.ProjectID
}
