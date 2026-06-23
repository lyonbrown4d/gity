// Package shared contains reusable HTTP endpoint helpers.
package shared

import (
	"strconv"
	"strings"

	setx "github.com/arcgolabs/collectionx/set"
)

func ParseIDFilter(raw string) *setx.Set[int64] {
	ids := setx.NewSet[int64]()
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return ids
	}
	for part := range strings.SplitSeq(raw, ",") {
		id, err := strconv.ParseInt(strings.TrimSpace(part), 10, 64)
		if err == nil && id > 0 {
			ids.Add(id)
		}
	}
	return ids
}

func FirstNonEmpty(values ...string) string {
	for _, value := range values {
		if strings.TrimSpace(value) != "" {
			return value
		}
	}
	return ""
}
