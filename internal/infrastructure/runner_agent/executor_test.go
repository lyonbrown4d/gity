package runneragent

import (
	"context"
	"encoding/json"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	"os"
	"os/exec"
	"path/filepath"
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
	}, cidomain.ProjectJob{
		ID:        46,
		ProjectID: 7,
		Kind:      "script",
		Payload:   string(payload),
		Attempts:  1,
	}, nil, nil, func(_ context.Context, _ cidomain.ProjectJob, _ ScriptPayload, workDir string) error {
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
	if err := os.MkdirAll(filepath.Dir(bareRepo), 0o750); err != nil {
		t.Fatalf("create bare repo parent: %v", err)
	}
	if err := runGit(ctx, filepath.Dir(bareRepo), "init", "--bare", bareRepo); err != nil {
		t.Fatalf("init bare repo: %v", err)
	}

	worktree := filepath.Join(t.TempDir(), "worktree")
	if err := os.MkdirAll(worktree, 0o750); err != nil {
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
	if err := os.WriteFile(filepath.Join(worktree, "README.md"), []byte("hello checkout\n"), 0o600); err != nil {
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
	}, cidomain.ProjectJob{
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
