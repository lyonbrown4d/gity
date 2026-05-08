package lfs

import (
	"strconv"
	"strings"

	"github.com/gofiber/fiber/v2"
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
