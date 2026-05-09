package runneragent_test

import (
	"encoding/json"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	runneragent "github.com/DaiYuANg/gity/internal/infrastructure/runner_agent"
	"os"
	"path/filepath"
	"testing"
)

func TestCollectArtifacts(t *testing.T) {
	t.Parallel()

	workDir := t.TempDir()
	if err := os.MkdirAll(filepath.Join(workDir, "dist"), 0o750); err != nil {
		t.Fatalf("mkdir dist: %v", err)
	}
	if err := os.WriteFile(filepath.Join(workDir, "dist", "report.txt"), []byte("report"), 0o600); err != nil {
		t.Fatalf("write report: %v", err)
	}
	payload, err := json.Marshal(runneragent.ScriptPayload{Artifacts: []string{"dist/**"}})
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}
	result, err := json.Marshal(runneragent.ScriptResult{WorkDir: workDir})
	if err != nil {
		t.Fatalf("marshal result: %v", err)
	}
	artifacts, err := runneragent.CollectArtifacts(cidomain.ProjectJob{Payload: string(payload)}, string(result))
	if err != nil {
		t.Fatalf("collect artifacts: %v", err)
	}
	if len(artifacts) != 1 {
		t.Fatalf("artifacts = %d", len(artifacts))
	}
	if artifacts[0].FilePath != "dist/report.txt" || string(artifacts[0].Content) != "report" {
		t.Fatalf("unexpected artifact: %+v", artifacts[0])
	}
}
