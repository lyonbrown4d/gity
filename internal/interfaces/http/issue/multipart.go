package issue

import (
	"fmt"
	"io"
	"mime"
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
			content, err := io.ReadAll(file)
			closeErr := file.Close()
			if err != nil {
				return fiber.NewError(http.StatusBadRequest, "read uploaded file failed")
			}
			if closeErr != nil {
				return fiber.NewError(http.StatusBadRequest, "close uploaded file failed")
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
