// Package mapperx contains mapper helpers.
package mapperx

import (
	"strings"
	"time"

	"github.com/arcgolabs/mapper"
)

func NewMapper() *mapper.Mapper {
	return mapper.New(
		mapper.WithFallbackTags("json"),
		mapper.Converter(func(value time.Time) string {
			if value.IsZero() {
				return ""
			}
			return value.UTC().Format(time.RFC3339)
		}),
		mapper.ConverterE(func(value string) (time.Time, error) {
			if strings.TrimSpace(value) == "" {
				return time.Time{}, nil
			}
			return time.Parse(time.RFC3339, value)
		}),
	)
}

func Ensure(instance *mapper.Mapper) *mapper.Mapper {
	if instance != nil {
		return instance
	}
	return NewMapper()
}
