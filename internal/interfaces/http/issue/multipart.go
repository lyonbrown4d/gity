package issue

import (
	"fmt"
	"io"
	"mime"
	"mime/multipart"
	"net/http"
	"net/url"
	"strconv"
	"strings"

	issueservice "github.com/DaiYuANg/gity/internal/application/issue"
	infraauth "github.com/DaiYuANg/gity/internal/infrastructure/auth"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
	"github.com/gofiber/fiber/v2"
	"github.com/samber/oops"
)

func RegisterMultipartRoutes(app *fiber.App, service *issueservice.Service, authRuntime *infraauth.Runtime) {
	routes := multipartRoutes{app: app, service: service, authRuntime: authRuntime}
	routes.register("/api/v1/repos")
	routes.register("/api/v1/projects")
}

type multipartRoutes struct {
	app         *fiber.App
	service     *issueservice.Service
	authRuntime *infraauth.Runtime
}

func (r multipartRoutes) register(prefix string) {
	r.app.Post(prefix+"/:id/issues/attachments", r.uploadAttachment)
	r.app.Get(prefix+"/:id/issues/attachments/raw", r.getDraftAttachmentRaw)
	r.app.Get(prefix+"/:id/issues/:issue_iid/attachments/:attachment_id/raw", r.getAttachmentRaw)
}

func (r multipartRoutes) uploadAttachment(c *fiber.Ctx) error {
	projectID, err := parseRequiredPathID(c, "id", "project")
	if err != nil {
		return err
	}
	issueIID, err := parseOptionalInt64(c.Query("issue_id"))
	if err != nil {
		return fiber.NewError(http.StatusBadRequest, "invalid issue_id")
	}
	uploadedByUserID, err := httpapi.ActorUserID(c.UserContext(), r.authRuntime, c.Get(fiber.HeaderAuthorization), 0)
	if err != nil {
		return err
	}
	fileHeader, content, err := readUploadFile(c)
	if err != nil {
		return err
	}
	uploaded, err := r.service.UploadAttachment(c.UserContext(), projectID, issueservice.AttachmentUploadInput{
		UploadedByUserID: uploadedByUserID,
		IssueIID:         issueIID,
		FileName:         fileHeader.Filename,
		ContentType:      strings.TrimSpace(fileHeader.Header.Get("Content-Type")),
		Content:          content,
	})
	if err != nil {
		return err
	}
	return c.Status(http.StatusCreated).JSON(toIssueAttachmentUploadView(projectID, issueIID, uploaded))
}

func (r multipartRoutes) getDraftAttachmentRaw(c *fiber.Ctx) error {
	projectID, err := parseRequiredPathID(c, "id", "project")
	if err != nil {
		return err
	}
	objectKey := c.Query("object_key")
	if strings.TrimSpace(objectKey) == "" {
		return fiber.NewError(http.StatusBadRequest, "object_key is required")
	}
	raw, err := r.service.GetDraftAttachmentRaw(c.UserContext(), projectID, objectKey)
	if err != nil {
		return err
	}
	return sendRawAttachment(c, raw)
}

func (r multipartRoutes) getAttachmentRaw(c *fiber.Ctx) error {
	projectID, issueIID, attachmentID, err := parseAttachmentPathIDs(c)
	if err != nil {
		return err
	}
	raw, err := r.service.GetAttachmentRaw(c.UserContext(), projectID, issueIID, attachmentID)
	if err != nil {
		return err
	}
	return sendRawAttachment(c, raw)
}

func readUploadFile(c *fiber.Ctx) (*multipart.FileHeader, []byte, error) {
	fileHeader, err := c.FormFile("file")
	if err != nil {
		return nil, nil, fiber.NewError(http.StatusBadRequest, "file is required")
	}
	file, err := fileHeader.Open()
	if err != nil {
		return nil, nil, fiber.NewError(http.StatusBadRequest, "open uploaded file failed")
	}
	content, err := io.ReadAll(file)
	closeErr := file.Close()
	if err != nil {
		return nil, nil, fiber.NewError(http.StatusBadRequest, "read uploaded file failed")
	}
	if closeErr != nil {
		return nil, nil, fiber.NewError(http.StatusBadRequest, "close uploaded file failed")
	}
	return fileHeader, content, nil
}

func parseAttachmentPathIDs(c *fiber.Ctx) (int64, int64, int64, error) {
	projectID, err := parseRequiredPathID(c, "id", "project")
	if err != nil {
		return 0, 0, 0, err
	}
	issueIID, err := parseRequiredPathID(c, "issue_iid", "issue")
	if err != nil {
		return 0, 0, 0, err
	}
	attachmentID, err := parseRequiredPathID(c, "attachment_id", "attachment")
	if err != nil {
		return 0, 0, 0, err
	}
	return projectID, issueIID, attachmentID, nil
}

func parseRequiredPathID(c *fiber.Ctx, paramName, label string) (int64, error) {
	id, err := strconv.ParseInt(c.Params(paramName), 10, 64)
	if err != nil || id <= 0 {
		return 0, fiber.NewError(http.StatusBadRequest, "invalid "+label+" id")
	}
	return id, nil
}

func parseOptionalInt64(value string) (int64, error) {
	value = strings.TrimSpace(value)
	if value == "" {
		return 0, nil
	}
	parsed, err := strconv.ParseInt(value, 10, 64)
	if err != nil {
		return 0, oops.In("http.issue").With("value", value).Wrapf(err, "parse optional int64")
	}
	if parsed < 0 {
		return 0, oops.In("http.issue").With("value", value).New("invalid int64")
	}
	return parsed, nil
}

func toIssueAttachmentUploadView(projectID, issueIID int64, item issueservice.AttachmentUploadView) issueAttachmentUploadView {
	downloadURL := fmt.Sprintf("/api/v1/projects/%d/issues/attachments/raw?object_key=%s", projectID, url.QueryEscape(item.ObjectKey))
	if item.Attachment != nil {
		downloadURL = fmt.Sprintf("/api/v1/projects/%d/issues/%d/attachments/%d/raw", projectID, issueIID, item.Attachment.ID)
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

func buildAttachmentMarkdown(fileName, contentType, downloadURL string) string {
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
