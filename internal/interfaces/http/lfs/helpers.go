package lfs

import (
	"net/http"
	"strconv"
	"strings"

	"github.com/gofiber/fiber/v3"
	"github.com/samber/oops"
)

func normalizeRequestPath(value string) string {
	return strings.Trim(strings.ReplaceAll(value, "\\", "/"), "/")
}

func parseLimit(value string) (int, error) {
	trimmed := strings.TrimSpace(value)
	if trimmed == "" {
		return 0, nil
	}
	parsed, err := strconv.Atoi(trimmed)
	if err != nil {
		return 0, oops.In("http.lfs").With("value", value).Wrapf(err, "parse lfs limit")
	}
	if parsed < 0 {
		return 0, oops.In("http.lfs").With("value", value, "limit", parsed).New("invalid limit")
	}
	return parsed, nil
}

func isLFSPath(path, method string) bool {
	normalized := normalizeRequestPath(path)
	if normalized == "" || !strings.Contains(normalized, ".git/") {
		return false
	}
	switch method {
	case fiber.MethodPost:
		return strings.HasSuffix(normalized, "/info/lfs/objects/batch") || strings.HasSuffix(normalized, "/info/lfs/locks") || strings.HasSuffix(normalized, "/info/lfs/locks/verify") || strings.HasSuffix(normalized, "/unlock")
	case fiber.MethodPut, fiber.MethodGet:
		return strings.Contains(normalized, "/info/lfs/objects/") || strings.HasSuffix(normalized, "/info/lfs/locks")
	default:
		return false
	}
}

func nextRoute(c fiber.Ctx) error {
	if err := c.Next(); err != nil {
		return oops.In("http.lfs").With("op", "next").Wrapf(err, "fiber next")
	}
	return nil
}

func sendJSON(c fiber.Ctx, status int, body any) error {
	if err := c.Status(status).JSON(body); err != nil {
		return oops.In("http.lfs").With("op", "json", "status", status).Wrapf(err, "fiber json")
	}
	return nil
}

func sendStatus(c fiber.Ctx, status int) error {
	if err := c.SendStatus(status); err != nil {
		return oops.In("http.lfs").With("op", "status", "status", status).Wrapf(err, "fiber send status")
	}
	return nil
}

func sendBytes(c fiber.Ctx, content []byte) error {
	if err := c.Send(content); err != nil {
		return oops.In("http.lfs").With("op", "send", "status", http.StatusOK).Wrapf(err, "fiber send")
	}
	return nil
}
