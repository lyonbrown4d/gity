package issue

import (
	"context"
	"fmt"
	"io"
	"mime"
	"net/http"
	"net/url"
	"strconv"
	"strings"

	"github.com/DaiYuANg/gity/internal/entity"
	"github.com/DaiYuANg/gity/internal/httpapi"
	platformauth "github.com/DaiYuANg/gity/internal/platform/auth"
	"github.com/DaiYuANg/gity/internal/platform/mapperx"
	issueservice "github.com/DaiYuANg/gity/internal/service/issue"
	"github.com/arcgolabs/httpx"
	"github.com/arcgolabs/mapper"
	"github.com/gofiber/fiber/v2"
)

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

type Endpoint struct {
	service     *issueservice.Service
	authRuntime *platformauth.Runtime
	mapper      *mapper.Mapper
}

func NewEndpoint(service *issueservice.Service, authRuntime *platformauth.Runtime, requestMapper *mapper.Mapper) *Endpoint {
	return &Endpoint{service: service, authRuntime: authRuntime, mapper: mapperx.Ensure(requestMapper)}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Issues", "Issues", "Project issue APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	service := e.service
	authRuntime := e.authRuntime

	listIssues := func(ctx context.Context, in *projectIssuesInput) (*issueOutput, error) {
		items, err := service.ListIssues(ctx, in.ProjectID)
		if err != nil {
			return nil, err
		}
		views := make([]issueView, 0, len(items))
		for _, item := range items {
			views = append(views, toIssueView(item))
		}
		return &issueOutput{Body: views}, nil
	}

	getIssue := func(ctx context.Context, in *projectIssueInput) (*issueOutput, error) {
		item, err := service.GetIssueByIID(ctx, in.ProjectID, in.IssueIID)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: toIssueView(item)}, nil
	}

	createIssue := func(ctx context.Context, in *createIssueInput) (*issueOutput, error) {
		input, err := mapperx.MapStrict[issueservice.CreateIssueInput](e.mapper, in.Body)
		if err != nil {
			return nil, err
		}
		authorUserID, err := httpapi.ActorUserID(ctx, authRuntime, in.Authorization, input.AuthorUserID)
		if err != nil {
			return nil, err
		}
		input.AuthorUserID = authorUserID
		item, err := service.CreateIssue(ctx, in.ProjectID, input)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: toIssueView(item)}, nil
	}

	updateIssue := func(ctx context.Context, in *updateIssueInput) (*issueOutput, error) {
		input, err := mapperx.MapStrict[issueservice.UpdateIssueInput](e.mapper, in.Body)
		if err != nil {
			return nil, err
		}
		if input.State == nil && in.Body.Status != nil {
			mapped := statusToState(*in.Body.Status)
			input.State = &mapped
		}
		item, err := service.UpdateIssue(ctx, in.ProjectID, in.IssueIID, input)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: toIssueView(item)}, nil
	}

	listComments := func(ctx context.Context, in *projectIssueInput) (*issueOutput, error) {
		items, err := service.ListComments(ctx, in.ProjectID, in.IssueIID)
		if err != nil {
			return nil, err
		}
		views := make([]issueCommentView, 0, len(items))
		for _, item := range items {
			views = append(views, toIssueCommentView(in.IssueIID, item))
		}
		return &issueOutput{Body: views}, nil
	}

	createComment := func(ctx context.Context, in *createCommentInput) (*issueOutput, error) {
		input, err := mapperx.MapStrict[issueservice.CreateCommentInput](e.mapper, in.Body)
		if err != nil {
			return nil, err
		}
		authorUserID, err := httpapi.ActorUserID(ctx, authRuntime, in.Authorization, input.AuthorUserID)
		if err != nil {
			return nil, err
		}
		input.AuthorUserID = authorUserID
		if input.Body == "" {
			input.Body = in.Body.Content
		}
		item, err := service.CreateComment(ctx, in.ProjectID, in.IssueIID, input)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: toIssueCommentView(in.IssueIID, item)}, nil
	}

	listAttachments := func(ctx context.Context, in *projectIssueInput) (*issueOutput, error) {
		items, err := service.ListAttachments(ctx, in.ProjectID, in.IssueIID)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: items}, nil
	}

	createAttachment := func(ctx context.Context, in *createAttachmentInput) (*issueOutput, error) {
		input, err := mapperx.MapStrict[issueservice.CreateAttachmentInput](e.mapper, in.Body)
		if err != nil {
			return nil, err
		}
		uploadedByUserID, err := httpapi.ActorUserID(ctx, authRuntime, in.Authorization, input.UploadedByUserID)
		if err != nil {
			return nil, err
		}
		input.UploadedByUserID = uploadedByUserID
		item, err := service.CreateAttachment(ctx, in.ProjectID, in.IssueIID, input)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: item}, nil
	}

	getAttachment := func(ctx context.Context, in *projectAttachmentInput) (*issueOutput, error) {
		item, err := service.GetAttachmentContent(ctx, in.ProjectID, in.IssueIID, in.AttachmentID)
		if err != nil {
			return nil, err
		}
		return &issueOutput{Body: item}, nil
	}

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/issues", listIssues),
		httpapi.Get("/repos/{id}/issues", listIssues, httpapi.DeprecatedRoute[projectIssuesInput, issueOutput]("Use GET /projects/{id}/issues instead.")),
		httpapi.Get("/projects/{id}/issues/{issue_iid}", getIssue),
		httpapi.Get("/repos/{id}/issues/{issue_iid}", getIssue, httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid} instead.")),
		httpapi.Get("/repos/{id}/issues/by-number/{issue_iid}", getIssue, httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid} instead.")),
		httpapi.Post("/projects/{id}/issues", createIssue, httpapi.RequireUserRoute[createIssueInput, issueOutput](authRuntime)),
		httpapi.Post("/repos/{id}/issues", createIssue,
			httpapi.RequireUserRoute[createIssueInput, issueOutput](authRuntime),
			httpapi.DeprecatedRoute[createIssueInput, issueOutput]("Use POST /projects/{id}/issues instead."),
		),
		httpapi.Patch("/projects/{id}/issues/{issue_iid}", updateIssue, httpapi.RequireUserRoute[updateIssueInput, issueOutput](authRuntime)),
		httpapi.Patch("/repos/{id}/issues/{issue_iid}", updateIssue,
			httpapi.RequireUserRoute[updateIssueInput, issueOutput](authRuntime),
			httpapi.DeprecatedRoute[updateIssueInput, issueOutput]("Use PATCH /projects/{id}/issues/{issue_iid} instead."),
		),
		httpapi.Get("/projects/{id}/issues/{issue_iid}/comments", listComments),
		httpapi.Get("/repos/{id}/issues/{issue_iid}/comments", listComments, httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid}/comments instead.")),
		httpapi.Post("/projects/{id}/issues/{issue_iid}/comments", createComment, httpapi.RequireUserRoute[createCommentInput, issueOutput](authRuntime)),
		httpapi.Post("/repos/{id}/issues/{issue_iid}/comments", createComment,
			httpapi.RequireUserRoute[createCommentInput, issueOutput](authRuntime),
			httpapi.DeprecatedRoute[createCommentInput, issueOutput]("Use POST /projects/{id}/issues/{issue_iid}/comments instead."),
		),
		httpapi.Get("/projects/{id}/issues/{issue_iid}/attachments", listAttachments),
		httpapi.Get("/repos/{id}/issues/{issue_iid}/attachments", listAttachments, httpapi.DeprecatedRoute[projectIssueInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid}/attachments instead.")),
		httpapi.Post("/projects/{id}/issues/{issue_iid}/attachments", createAttachment, httpapi.RequireUserRoute[createAttachmentInput, issueOutput](authRuntime)),
		httpapi.Post("/repos/{id}/issues/{issue_iid}/attachments", createAttachment,
			httpapi.RequireUserRoute[createAttachmentInput, issueOutput](authRuntime),
			httpapi.DeprecatedRoute[createAttachmentInput, issueOutput]("Use POST /projects/{id}/issues/{issue_iid}/attachments instead."),
		),
		httpapi.Get("/projects/{id}/issues/{issue_iid}/attachments/{attachment_id}", getAttachment),
		httpapi.Get("/repos/{id}/issues/{issue_iid}/attachments/{attachment_id}", getAttachment, httpapi.DeprecatedRoute[projectAttachmentInput, issueOutput]("Use GET /projects/{id}/issues/{issue_iid}/attachments/{attachment_id} instead.")),
	)
}

func RegisterMultipartRoutes(app *fiber.App, service *issueservice.Service, authRuntime *platformauth.Runtime) {
	registerMultipartUpload := func(prefix string) {
		app.Post(prefix+"/:id/issues/attachments", func(c *fiber.Ctx) error {
			projectID, err := strconv.ParseInt(c.Params("id"), 10, 64)
			if err != nil || projectID <= 0 {
				return fiber.NewError(http.StatusBadRequest, "invalid project id")
			}
			issueIID, err := parseOptionalInt64(c.Query("issue_id"))
			if err != nil {
				return fiber.NewError(http.StatusBadRequest, "invalid issue_id")
			}
			uploadedByUserID, err := httpapi.ActorUserID(c.UserContext(), authRuntime, c.Get(fiber.HeaderAuthorization), 0)
			if err != nil {
				return err
			}
			fileHeader, err := c.FormFile("file")
			if err != nil {
				return fiber.NewError(http.StatusBadRequest, "file is required")
			}
			file, err := fileHeader.Open()
			if err != nil {
				return fiber.NewError(http.StatusBadRequest, "open uploaded file failed")
			}
			defer file.Close()
			content, err := io.ReadAll(file)
			if err != nil {
				return fiber.NewError(http.StatusBadRequest, "read uploaded file failed")
			}
			contentType := strings.TrimSpace(fileHeader.Header.Get("Content-Type"))
			uploaded, err := service.UploadAttachment(c.UserContext(), projectID, issueservice.AttachmentUploadInput{
				UploadedByUserID: uploadedByUserID,
				IssueIID:         issueIID,
				FileName:         fileHeader.Filename,
				ContentType:      contentType,
				Content:          content,
			})
			if err != nil {
				return err
			}
			return c.Status(http.StatusCreated).JSON(toIssueAttachmentUploadView(projectID, issueIID, uploaded))
		})

		app.Get(prefix+"/:id/issues/attachments/raw", func(c *fiber.Ctx) error {
			projectID, err := strconv.ParseInt(c.Params("id"), 10, 64)
			if err != nil || projectID <= 0 {
				return fiber.NewError(http.StatusBadRequest, "invalid project id")
			}
			objectKey := c.Query("object_key")
			if strings.TrimSpace(objectKey) == "" {
				return fiber.NewError(http.StatusBadRequest, "object_key is required")
			}
			raw, err := service.GetDraftAttachmentRaw(c.UserContext(), projectID, objectKey)
			if err != nil {
				return err
			}
			return sendRawAttachment(c, raw)
		})

		app.Get(prefix+"/:id/issues/:issue_iid/attachments/:attachment_id/raw", func(c *fiber.Ctx) error {
			projectID, err := strconv.ParseInt(c.Params("id"), 10, 64)
			if err != nil || projectID <= 0 {
				return fiber.NewError(http.StatusBadRequest, "invalid project id")
			}
			issueIID, err := strconv.ParseInt(c.Params("issue_iid"), 10, 64)
			if err != nil || issueIID <= 0 {
				return fiber.NewError(http.StatusBadRequest, "invalid issue id")
			}
			attachmentID, err := strconv.ParseInt(c.Params("attachment_id"), 10, 64)
			if err != nil || attachmentID <= 0 {
				return fiber.NewError(http.StatusBadRequest, "invalid attachment id")
			}
			raw, err := service.GetAttachmentRaw(c.UserContext(), projectID, issueIID, attachmentID)
			if err != nil {
				return err
			}
			return sendRawAttachment(c, raw)
		})
	}

	registerMultipartUpload("/api/v1/repos")
	registerMultipartUpload("/api/v1/projects")
}

func toIssueView(item entity.ProjectIssue) issueView {
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

func toIssueCommentView(issueIID int64, item entity.ProjectIssueComment) issueCommentView {
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

func parseOptionalInt64(value string) (int64, error) {
	value = strings.TrimSpace(value)
	if value == "" {
		return 0, nil
	}
	parsed, err := strconv.ParseInt(value, 10, 64)
	if err != nil || parsed < 0 {
		return 0, fmt.Errorf("invalid int64")
	}
	return parsed, nil
}

func toIssueAttachmentUploadView(projectID int64, issueIID int64, item issueservice.AttachmentUploadView) issueAttachmentUploadView {
	downloadURL := fmt.Sprintf("/api/v1/repos/%d/issues/attachments/raw?object_key=%s", projectID, url.QueryEscape(item.ObjectKey))
	if item.Attachment != nil {
		downloadURL = fmt.Sprintf("/api/v1/repos/%d/issues/%d/attachments/%d/raw", projectID, issueIID, item.Attachment.ID)
	}
	return issueAttachmentUploadView{
		URL:         downloadURL,
		ObjectKey:   item.ObjectKey,
		FileName:    item.FileName,
		ContentType: item.ContentType,
		Size:        item.ByteSize,
		Markdown:    buildAttachmentMarkdown(item.FileName, item.ContentType, downloadURL),
	}
}

func buildAttachmentMarkdown(fileName string, contentType string, downloadURL string) string {
	escapedName := strings.ReplaceAll(strings.TrimSpace(fileName), "]", "\\]")
	if escapedName == "" {
		escapedName = "attachment"
	}
	if strings.HasPrefix(strings.ToLower(strings.TrimSpace(contentType)), "image/") {
		return fmt.Sprintf("![%s](%s)", escapedName, downloadURL)
	}
	return fmt.Sprintf("[%s](%s)", escapedName, downloadURL)
}

func sendRawAttachment(c *fiber.Ctx, raw issueservice.AttachmentRawContent) error {
	contentType := strings.TrimSpace(raw.ContentType)
	if contentType == "" {
		contentType = "application/octet-stream"
	}
	c.Set(fiber.HeaderContentType, contentType)
	if strings.TrimSpace(raw.FileName) != "" {
		c.Set(fiber.HeaderContentDisposition, mime.FormatMediaType("inline", map[string]string{"filename": raw.FileName}))
	}
	return c.Send(raw.Content)
}
