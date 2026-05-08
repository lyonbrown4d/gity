package issue

type projectIssueInput struct {
	ProjectID int64 `path:"id"`
	IssueIID  int64 `path:"issue_iid"`
}

type projectIssuesInput struct {
	ProjectID int64 `path:"id"`
}

type projectAttachmentInput struct {
	ProjectID    int64 `path:"id"`
	IssueIID     int64 `path:"issue_iid"`
	AttachmentID int64 `path:"attachment_id"`
}

type createIssueInput struct {
	ProjectID     int64           `path:"id"`
	Authorization string          `header:"Authorization"`
	Body          createIssueBody `json:"body"`
}

type updateIssueInput struct {
	ProjectID     int64           `path:"id"`
	IssueIID      int64           `path:"issue_iid"`
	Authorization string          `header:"Authorization"`
	Body          updateIssueBody `json:"body"`
}

type createCommentInput struct {
	ProjectID     int64             `path:"id"`
	IssueIID      int64             `path:"issue_iid"`
	Authorization string            `header:"Authorization"`
	Body          createCommentBody `json:"body"`
}

type createAttachmentInput struct {
	ProjectID     int64                `path:"id"`
	IssueIID      int64                `path:"issue_iid"`
	Authorization string               `header:"Authorization"`
	Body          createAttachmentBody `json:"body"`
}

type issueOutput struct {
	Body any `json:"body"`
}

type createIssueBody struct {
	AuthorUserID int64  `json:"author_user_id"`
	Title        string `json:"title"`
	Description  string `json:"description"`
}

type updateIssueBody struct {
	Title       *string `json:"title"`
	Description *string `json:"description"`
	State       *string `json:"state"`
	Status      *string `json:"status"`
}

type createCommentBody struct {
	AuthorUserID int64  `json:"author_user_id"`
	Body         string `json:"body"`
	Content      string `json:"content"`
}

type createAttachmentBody struct {
	UploadedByUserID int64  `json:"uploaded_by_user_id"`
	FileName         string `json:"file_name"`
	ContentType      string `json:"content_type"`
	ContentBase64    string `json:"content_base64"`
}

type issueView struct {
	ID             string  `json:"id"`
	RepositoryID   string  `json:"repository_id"`
	Number         int64   `json:"number"`
	Title          string  `json:"title"`
	Description    string  `json:"description"`
	Status         string  `json:"status"`
	AuthorUserID   string  `json:"author_user_id"`
	AssigneeUserID *string `json:"assignee_user_id,omitempty"`
	CreatedAt      string  `json:"created_at"`
	UpdatedAt      string  `json:"updated_at"`
	ClosedAt       *string `json:"closed_at,omitempty"`
}

type issueCommentView struct {
	ID           string `json:"id"`
	IssueID      string `json:"issue_id"`
	AuthorUserID string `json:"author_user_id"`
	Content      string `json:"content"`
	CreatedAt    string `json:"created_at"`
	UpdatedAt    string `json:"updated_at"`
}

type issueAttachmentUploadView struct {
	URL         string `json:"url"`
	ObjectKey   string `json:"object_key"`
	FileName    string `json:"file_name"`
	ContentType string `json:"content_type"`
	Size        int64  `json:"size"`
	Markdown    string `json:"markdown"`
}
