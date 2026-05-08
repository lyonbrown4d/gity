package runneragent

import (
	"context"
	"encoding/json"
	"runtime"
	"strings"
	"testing"
	"time"

	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
)

func TestExecuteScriptJobFailure(t *testing.T) {
	t.Parallel()

	script := []string{"echo boom", "exit 7"}
	if runtime.GOOS == "windows" {
		script = []string{`Write-Output "boom"`, "exit 7"}
	}
	payload, err := json.Marshal(ScriptPayload{
		Script:         script,
		TimeoutSeconds: 5,
	})
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}

	resultJSON, err := ExecuteScriptJob(context.Background(), Config{
		WorkDir:        t.TempDir(),
		LeaseSeconds:   30,
		MaxOutputBytes: 1024,
	}, cidomain.ProjectJob{
		ID:        43,
		ProjectID: 7,
		Kind:      "script",
		Payload:   string(payload),
		Attempts:  1,
	})
	if err == nil {
		t.Fatalf("expected script failure")
	}
	var result ScriptResult
	if err := json.Unmarshal([]byte(resultJSON), &result); err != nil {
		t.Fatalf("unmarshal result: %v", err)
	}
	if result.ExitCode != 7 || !strings.Contains(result.Output, "boom") {
		t.Fatalf("unexpected failure result: %+v", result)
	}
}

func TestExecuteScriptJobCancellation(t *testing.T) {
	t.Parallel()

	script := []string{"sleep 5"}
	if runtime.GOOS == "windows" {
		script = []string{`Start-Sleep -Seconds 5`}
	}
	payload, err := json.Marshal(ScriptPayload{
		Script:         script,
		TimeoutSeconds: 10,
	})
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}

	cancelAt := time.Now().Add(1200 * time.Millisecond)
	resultJSON, err := ExecuteScriptJobWithChecker(context.Background(), Config{
		WorkDir:        t.TempDir(),
		LeaseSeconds:   30,
		MaxOutputBytes: 1024,
	}, cidomain.ProjectJob{
		ID:        101,
		ProjectID: 7,
		Kind:      "script",
		Payload:   string(payload),
		Attempts:  1,
	}, func(_ context.Context) (bool, error) {
		if time.Now().After(cancelAt) {
			return true, nil
		}
		return false, nil
	})
	if err == nil {
		t.Fatalf("expected script cancellation")
	}
	var result ScriptResult
	if err := json.Unmarshal([]byte(resultJSON), &result); err != nil {
		t.Fatalf("unmarshal result: %v", err)
	}
	if result.ExitCode != -1 {
		t.Fatalf("unexpected cancel result: %+v", result)
	}
}

func TestConfigFromEnvArgs(t *testing.T) {
	t.Setenv("GITY_RUNNER_TOKEN", "from-env")
	t.Setenv("GITY_RUNNER_URL", "http://gity.local/v1")

	cfg, err := ConfigFromEnv([]string{"-token", "from-flag", "-poll-interval", "2s"})
	if err != nil {
		t.Fatalf("config from env args: %v", err)
	}
	if cfg.Token != "from-flag" || cfg.ServerURL != "http://gity.local/v1" || cfg.PollInterval != 2*time.Second {
		t.Fatalf("unexpected config: %+v", cfg)
	}
}
