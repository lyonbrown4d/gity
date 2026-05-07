package mapperx

import (
	"testing"
	"time"
)

type sourceDTO struct {
	RefName   string    `json:"ref_name"`
	CreatedAt time.Time `json:"created_at"`
}

type targetDTO struct {
	RefName   string `json:"ref_name"`
	CreatedAt string `json:"created_at"`
}

func TestMapStrictUsesJSONFallbackTagsAndConverters(t *testing.T) {
	createdAt := time.Date(2026, 5, 7, 12, 0, 0, 0, time.FixedZone("CST", 8*60*60))

	target, err := MapStrict[targetDTO](NewMapper(), sourceDTO{
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
