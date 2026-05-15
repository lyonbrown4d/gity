package runneragent_test

import (
	"context"
	"encoding/json"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	runneragent "github.com/lyonbrown4d/gity/internal/infrastructure/runner_agent"
	"os"
	"runtime"
	"strings"
	"sync"
	"testing"
)

func TestExecuteScriptJob(t *testing.T) {
	t.Parallel()

	script := []string{"echo hello"}
	if runtime.GOOS == "windows" {
		script = []string{`Write-Output "hello"`}
	}
	payload, err := json.Marshal(runneragent.ScriptPayload{
		Script:         script,
		TimeoutSeconds: 5,
	})
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}

	resultJSON, err := runneragent.ExecuteScriptJob(context.Background(), runneragent.Config{
		WorkDir:        t.TempDir(),
		LeaseSeconds:   30,
		MaxOutputBytes: 1024,
	}, cidomain.ProjectJob{
		ID:        42,
		ProjectID: 7,
		Kind:      "script",
		Payload:   string(payload),
		Attempts:  1,
	})
	if err != nil {
		t.Fatalf("execute script job: %v", err)
	}
	var result runneragent.ScriptResult
	if err := json.Unmarshal([]byte(resultJSON), &result); err != nil {
		t.Fatalf("unmarshal result: %v", err)
	}
	if result.ExitCode != 0 || !strings.Contains(result.Output, "hello") || result.WorkDir == "" {
		t.Fatalf("unexpected result: %+v", result)
	}
}

func TestExecuteScriptJobStreamsTrace(t *testing.T) {
	t.Parallel()

	script := []string{"echo alpha", "echo beta"}
	if runtime.GOOS == "windows" {
		script = []string{`Write-Output "alpha"`, `Write-Output "beta"`}
	}
	payload, err := json.Marshal(runneragent.ScriptPayload{
		Script:         script,
		TimeoutSeconds: 5,
	})
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}

	var mu sync.Mutex
	chunks := make([]string, 0)
	resultJSON, err := runneragent.ExecuteScriptJobWithTrace(context.Background(), runneragent.Config{
		WorkDir:        t.TempDir(),
		LeaseSeconds:   30,
		MaxOutputBytes: 1024,
	}, cidomain.ProjectJob{
		ID:        45,
		ProjectID: 7,
		Kind:      "script",
		Payload:   string(payload),
		Attempts:  1,
	}, nil, func(_ context.Context, output string, _ bool, _ int64) error {
		mu.Lock()
		defer mu.Unlock()
		chunks = append(chunks, output)
		return nil
	})
	if err != nil {
		t.Fatalf("execute script job with trace: %v", err)
	}
	var result runneragent.ScriptResult
	if err := json.Unmarshal([]byte(resultJSON), &result); err != nil {
		t.Fatalf("unmarshal result: %v", err)
	}
	mu.Lock()
	streamed := strings.Join(chunks, "")
	mu.Unlock()
	if !strings.Contains(streamed, "alpha") || !strings.Contains(streamed, "beta") {
		t.Fatalf("expected streamed output, got %q", streamed)
	}
	if result.Output != streamed {
		t.Fatalf("streamed output should match captured output: streamed=%q captured=%q", streamed, result.Output)
	}
}

func TestExecuteScriptJobMasksSecrets(t *testing.T) {
	t.Parallel()

	script := []string{`printf "token=super-secret-token\n"`}
	if runtime.GOOS == "windows" {
		script = []string{`Write-Output "token=super-secret-token"`}
	}
	payload, err := json.Marshal(runneragent.ScriptPayload{
		Script:         script,
		MaskedValues:   []string{"super-secret-token"},
		TimeoutSeconds: 5,
	})
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}
	streamed := ""
	resultJSON, err := runneragent.ExecuteScriptJobWithTrace(context.Background(), runneragent.Config{
		WorkDir:        t.TempDir(),
		LeaseSeconds:   30,
		MaxOutputBytes: 1024,
	}, cidomain.ProjectJob{
		ID:        47,
		ProjectID: 7,
		Kind:      "script",
		Payload:   string(payload),
		Attempts:  1,
	}, nil, func(_ context.Context, output string, _ bool, _ int64) error {
		streamed += output
		return nil
	})
	if err != nil {
		t.Fatalf("execute masked script job: %v", err)
	}
	var result runneragent.ScriptResult
	if err := json.Unmarshal([]byte(resultJSON), &result); err != nil {
		t.Fatalf("unmarshal result: %v", err)
	}
	if strings.Contains(result.Output, "super-secret-token") || strings.Contains(streamed, "super-secret-token") {
		t.Fatalf("secret should be masked: result=%q streamed=%q", result.Output, streamed)
	}
	if !strings.Contains(result.Output, "[MASKED]") || !strings.Contains(streamed, "[MASKED]") {
		t.Fatalf("masked marker missing: result=%q streamed=%q", result.Output, streamed)
	}
}

func TestExecuteScriptJobRejectsDisallowedShell(t *testing.T) {
	t.Parallel()

	payload, err := json.Marshal(runneragent.ScriptPayload{
		Shell:          "python",
		Script:         []string{`print("nope")`},
		TimeoutSeconds: 5,
	})
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}
	_, err = runneragent.ExecuteScriptJob(context.Background(), runneragent.Config{
		WorkDir:        t.TempDir(),
		AllowedShells:  []string{"sh"},
		LeaseSeconds:   30,
		MaxOutputBytes: 1024,
	}, cidomain.ProjectJob{
		ID:        48,
		ProjectID: 7,
		Kind:      "script",
		Payload:   string(payload),
		Attempts:  1,
	})
	if err == nil || !strings.Contains(err.Error(), "not allowed") {
		t.Fatalf("expected disallowed shell error, got %v", err)
	}
}

func TestExecuteScriptJobCleansWorkspace(t *testing.T) {
	t.Parallel()

	script := []string{"echo hello"}
	if runtime.GOOS == "windows" {
		script = []string{`Write-Output "hello"`}
	}
	payload, err := json.Marshal(runneragent.ScriptPayload{
		Script:         script,
		TimeoutSeconds: 5,
	})
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}
	resultJSON, err := runneragent.ExecuteScriptJob(context.Background(), runneragent.Config{
		WorkDir:        t.TempDir(),
		LeaseSeconds:   30,
		MaxOutputBytes: 1024,
		CleanWorkspace: true,
	}, cidomain.ProjectJob{
		ID:        49,
		ProjectID: 7,
		Kind:      "script",
		Payload:   string(payload),
		Attempts:  1,
	})
	if err != nil {
		t.Fatalf("execute script job: %v", err)
	}
	var result runneragent.ScriptResult
	if err := json.Unmarshal([]byte(resultJSON), &result); err != nil {
		t.Fatalf("unmarshal result: %v", err)
	}
	if _, statErr := os.Stat(result.WorkDir); !os.IsNotExist(statErr) {
		t.Fatalf("expected workspace to be removed, stat error=%v", statErr)
	}
}

func TestExecuteScriptJobDownloadsSourceArchive(t *testing.T) {
	t.Parallel()

	script := []string{"cat README.md"}
	if runtime.GOOS == "windows" {
		script = []string{`Get-Content README.md`}
	}
	payload, err := json.Marshal(runneragent.ScriptPayload{
		ProjectFullPath: "core/gity",
		RefName:         "main",
		Script:          script,
		TimeoutSeconds:  5,
	})
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}
	archive := testSourceArchive(t, map[string]string{"README.md": "hello remote source\n"})

	resultJSON, err := runneragent.ExecuteScriptJobWithSource(context.Background(), runneragent.Config{
		WorkDir:        t.TempDir(),
		LeaseSeconds:   30,
		MaxOutputBytes: 1024,
	}, cidomain.ProjectJob{
		ID:        46,
		ProjectID: 7,
		Kind:      "script",
		Payload:   string(payload),
		Attempts:  1,
	}, nil, nil, func(_ context.Context, _ cidomain.ProjectJob, _ runneragent.ScriptPayload, workDir string) error {
		return runneragent.ExtractSourceArchive(archive, workDir)
	})
	if err != nil {
		t.Fatalf("execute remote source script job: %v", err)
	}
	var result runneragent.ScriptResult
	if err := json.Unmarshal([]byte(resultJSON), &result); err != nil {
		t.Fatalf("unmarshal result: %v", err)
	}
	if !strings.Contains(result.Output, "hello remote source") {
		t.Fatalf("expected remote source content in output: %+v", result)
	}
}
