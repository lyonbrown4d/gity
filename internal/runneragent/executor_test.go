package runneragent

import (
	"archive/zip"
	"bytes"
	"context"
	"encoding/json"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/DaiYuANg/gity/internal/entity"
)

func TestExecuteScriptJob(t *testing.T) {
	t.Parallel()

	script := []string{"echo hello"}
	if runtime.GOOS == "windows" {
		script = []string{`Write-Output "hello"`}
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
	}, entity.ProjectJob{
		ID:        42,
		ProjectID: 7,
		Kind:      "script",
		Payload:   string(payload),
		Attempts:  1,
	})
	if err != nil {
		t.Fatalf("execute script job: %v", err)
	}
	var result ScriptResult
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
	payload, err := json.Marshal(ScriptPayload{
		Script:         script,
		TimeoutSeconds: 5,
	})
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}

	var mu sync.Mutex
	chunks := make([]string, 0)
	resultJSON, err := ExecuteScriptJobWithTrace(context.Background(), Config{
		WorkDir:        t.TempDir(),
		LeaseSeconds:   30,
		MaxOutputBytes: 1024,
	}, entity.ProjectJob{
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
	var result ScriptResult
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

func TestExecuteScriptJobDownloadsSourceArchive(t *testing.T) {
	t.Parallel()

	script := []string{"cat README.md"}
	if runtime.GOOS == "windows" {
		script = []string{`Get-Content README.md`}
	}
	payload, err := json.Marshal(ScriptPayload{
		ProjectFullPath: "core/gity",
		RefName:         "main",
		Script:          script,
		TimeoutSeconds:  5,
	})
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}
	archive := testSourceArchive(t, map[string]string{"README.md": "hello remote source\n"})

	resultJSON, err := ExecuteScriptJobWithSource(context.Background(), Config{
		WorkDir:        t.TempDir(),
		LeaseSeconds:   30,
		MaxOutputBytes: 1024,
	}, entity.ProjectJob{
		ID:        46,
		ProjectID: 7,
		Kind:      "script",
		Payload:   string(payload),
		Attempts:  1,
	}, nil, nil, func(_ context.Context, _ entity.ProjectJob, _ ScriptPayload, workDir string) error {
		return ExtractSourceArchive(archive, workDir)
	})
	if err != nil {
		t.Fatalf("execute remote source script job: %v", err)
	}
	var result ScriptResult
	if err := json.Unmarshal([]byte(resultJSON), &result); err != nil {
		t.Fatalf("unmarshal result: %v", err)
	}
	if !strings.Contains(result.Output, "hello remote source") {
		t.Fatalf("expected remote source content in output: %+v", result)
	}
}

func TestExecuteScriptJobChecksOutLocalRepository(t *testing.T) {
	t.Parallel()

	if _, err := exec.LookPath("git"); err != nil {
		t.Skip("git is not available")
	}
	ctx := context.Background()
	repoRoot := t.TempDir()
	bareRepo := filepath.Join(repoRoot, "core", "gity.git")
	if err := os.MkdirAll(filepath.Dir(bareRepo), 0o755); err != nil {
		t.Fatalf("create bare repo parent: %v", err)
	}
	if err := runGit(ctx, filepath.Dir(bareRepo), "init", "--bare", bareRepo); err != nil {
		t.Fatalf("init bare repo: %v", err)
	}

	worktree := filepath.Join(t.TempDir(), "worktree")
	if err := os.MkdirAll(worktree, 0o755); err != nil {
		t.Fatalf("create worktree: %v", err)
	}
	if err := runGit(ctx, worktree, "init", "-b", "main"); err != nil {
		t.Fatalf("init worktree: %v", err)
	}
	if err := runGit(ctx, worktree, "config", "user.name", "Gity Test"); err != nil {
		t.Fatalf("config user name: %v", err)
	}
	if err := runGit(ctx, worktree, "config", "user.email", "test@gity.dev"); err != nil {
		t.Fatalf("config user email: %v", err)
	}
	if err := os.WriteFile(filepath.Join(worktree, "README.md"), []byte("hello checkout\n"), 0o644); err != nil {
		t.Fatalf("write readme: %v", err)
	}
	if err := runGit(ctx, worktree, "add", "README.md"); err != nil {
		t.Fatalf("add readme: %v", err)
	}
	if err := runGit(ctx, worktree, "commit", "-m", "Add README"); err != nil {
		t.Fatalf("commit readme: %v", err)
	}
	if err := runGit(ctx, worktree, "push", bareRepo, "HEAD:refs/heads/main"); err != nil {
		t.Fatalf("push readme: %v", err)
	}

	script := []string{"cat README.md"}
	if runtime.GOOS == "windows" {
		script = []string{`Get-Content README.md`}
	}
	payload, err := json.Marshal(ScriptPayload{
		ProjectFullPath: "core/gity",
		RefName:         "main",
		Script:          script,
		TimeoutSeconds:  5,
	})
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}
	resultJSON, err := ExecuteScriptJob(ctx, Config{
		WorkDir:        t.TempDir(),
		RepoRoot:       repoRoot,
		LeaseSeconds:   30,
		MaxOutputBytes: 1024,
	}, entity.ProjectJob{
		ID:        44,
		ProjectID: 7,
		Kind:      "script",
		Payload:   string(payload),
		Attempts:  1,
	})
	if err != nil {
		t.Fatalf("execute checkout script job: %v", err)
	}
	var result ScriptResult
	if err := json.Unmarshal([]byte(resultJSON), &result); err != nil {
		t.Fatalf("unmarshal result: %v", err)
	}
	if !strings.Contains(result.Output, "hello checkout") {
		t.Fatalf("expected repository content in output: %+v", result)
	}
}

func testSourceArchive(t *testing.T, files map[string]string) []byte {
	t.Helper()
	var buffer bytes.Buffer
	writer := zip.NewWriter(&buffer)
	for name, content := range files {
		file, err := writer.Create(name)
		if err != nil {
			t.Fatalf("create archive file: %v", err)
		}
		if _, err := file.Write([]byte(content)); err != nil {
			t.Fatalf("write archive file: %v", err)
		}
	}
	if err := writer.Close(); err != nil {
		t.Fatalf("close archive: %v", err)
	}
	return buffer.Bytes()
}

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
	}, entity.ProjectJob{
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
	}, entity.ProjectJob{
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
