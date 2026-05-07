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

func Map[T any](instance *mapper.Mapper, source any, opts ...mapper.Option) (T, error) {
	var target T
	err := Ensure(instance).MapInto(&target, source, opts...)
	return target, err
}

func MapStrict[T any](instance *mapper.Mapper, source any) (T, error) {
	return Map[T](instance, source, mapper.Strict())
}
