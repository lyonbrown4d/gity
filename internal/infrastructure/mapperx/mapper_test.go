package mapperx_test

import (
	"testing"
	"time"

	"github.com/arcgolabs/mapper"
	"github.com/lyonbrown4d/gity/internal/infrastructure/mapperx"
)

type sourceDTO struct {
	RefName   string    `json:"ref_name"`
	CreatedAt time.Time `json:"created_at"`
}

type targetDTO struct {
	RefName   string `json:"ref_name"`
	CreatedAt string `json:"created_at"`
}

type timeSourceDTO struct {
	RunAfter string `json:"run_after"`
}

type timeTargetDTO struct {
	RunAfter time.Time `json:"run_after"`
}

func TestNewMapperMapUsesJSONFallbackTagsAndConverters(t *testing.T) {
	createdAt := time.Date(2026, 5, 7, 12, 0, 0, 0, time.FixedZone("CST", 8*60*60))

	target, err := mapperx.NewMapper().Map[targetDTO, sourceDTO](sourceDTO{
		RefName:   "main",
		CreatedAt: createdAt,
	}, mapper.Strict())
	if err != nil {
		t.Fatalf("map: %v", err)
	}
	if target.RefName != "main" {
		t.Fatalf("ref name = %q", target.RefName)
	}
	if target.CreatedAt != "2026-05-07T04:00:00Z" {
		t.Fatalf("created at = %q", target.CreatedAt)
	}
}

func TestNewMapperMapIntoParsesRFC3339Time(t *testing.T) {
	var target timeTargetDTO
	err := mapperx.NewMapper().MapInto[timeTargetDTO, timeSourceDTO](
		&target,
		timeSourceDTO{RunAfter: "2026-05-07T04:00:00Z"},
		mapper.Strict(),
	)
	if err != nil {
		t.Fatalf("map into: %v", err)
	}
	if target.RunAfter.UTC().Format(time.RFC3339) != "2026-05-07T04:00:00Z" {
		t.Fatalf("run after = %s", target.RunAfter.UTC().Format(time.RFC3339))
	}
}

func TestNewMapperMapStrictRejectsUnmatchedDestinationField(t *testing.T) {
	type strictTargetDTO struct {
		RefName string `json:"ref_name"`
		Missing string `json:"missing"`
	}

	_, err := mapperx.NewMapper().Map[strictTargetDTO, sourceDTO](sourceDTO{RefName: "main"}, mapper.Strict())
	if err == nil {
		t.Fatal("map strict should reject an unmatched destination field")
	}
}
