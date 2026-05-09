package mapperx_test

import (
	"testing"
	"time"

	"github.com/DaiYuANg/gity/internal/infrastructure/mapperx"
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

func TestMapStrictUsesJSONFallbackTagsAndConverters(t *testing.T) {
	createdAt := time.Date(2026, 5, 7, 12, 0, 0, 0, time.FixedZone("CST", 8*60*60))

	target, err := mapperx.MapStrict[targetDTO](mapperx.NewMapper(), sourceDTO{
		RefName:   "main",
		CreatedAt: createdAt,
	})
	if err != nil {
		t.Fatalf("map strict: %v", err)
	}
	if target.RefName != "main" {
		t.Fatalf("ref name = %q", target.RefName)
	}
	if target.CreatedAt != "2026-05-07T04:00:00Z" {
		t.Fatalf("created at = %q", target.CreatedAt)
	}
}

func TestMapStrictParsesRFC3339Time(t *testing.T) {
	target, err := mapperx.MapStrict[timeTargetDTO](mapperx.NewMapper(), timeSourceDTO{RunAfter: "2026-05-07T04:00:00Z"})
	if err != nil {
		t.Fatalf("map strict: %v", err)
	}
	if target.RunAfter.UTC().Format(time.RFC3339) != "2026-05-07T04:00:00Z" {
		t.Fatalf("run after = %s", target.RunAfter.UTC().Format(time.RFC3339))
	}
}
