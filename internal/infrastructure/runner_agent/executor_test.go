package runneragent_test

import (
	"context"
	"encoding/json"
	"fmt"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	runneragent "github.com/lyonbrown4d/gity/internal/infrastructure/runner_agent"
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

func TestExecuteScriptJobChecksOutLocalRepository(t *testing.T) {
	t.Parallel()

	if _, err := exec.LookPath("git"); err != nil {
		t.Skip("git is not available")
	}
	ctx := context.Background()
	repoRoot := prepareLocalRepository(ctx, t)

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
	resultJSON, err := runneragent.ExecuteScriptJob(ctx, runneragent.Config{
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
	var result runneragent.ScriptResult
	if err := json.Unmarshal([]byte(resultJSON), &result); err != nil {
		t.Fatalf("unmarshal result: %v", err)
	}
	if !strings.Contains(result.Output, "hello checkout") {
		t.Fatalf("expected repository content in output: %+v", result)
	}
}

func prepareLocalRepository(ctx context.Context, t *testing.T) string {
	t.Helper()
	repoRoot := t.TempDir()
	bareRepo := filepath.Join(repoRoot, "core", "gity.git")
	createBareRepository(ctx, t, bareRepo)
	pushReadmeFixture(ctx, t, bareRepo)
	return repoRoot
}

func createBareRepository(ctx context.Context, t *testing.T, bareRepo string) {
	t.Helper()
	if err := os.MkdirAll(filepath.Dir(bareRepo), 0o750); err != nil {
		t.Fatalf("create bare repo parent: %v", err)
	}
	if err := runGit(ctx, filepath.Dir(bareRepo), "init", "--bare", bareRepo); err != nil {
		t.Fatalf("init bare repo: %v", err)
	}
}

func pushReadmeFixture(ctx context.Context, t *testing.T, bareRepo string) {
	t.Helper()
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
	writeReadmeFixture(ctx, t, worktree)
	if err := runGit(ctx, worktree, "push", bareRepo, "HEAD:refs/heads/main"); err != nil {
		t.Fatalf("push readme: %v", err)
	}
}

func writeReadmeFixture(ctx context.Context, t *testing.T, worktree string) {
	t.Helper()
	if err := os.WriteFile(filepath.Join(worktree, "README.md"), []byte("hello checkout\n"), 0o600); err != nil {
		t.Fatalf("write readme: %v", err)
	}
	if err := runGit(ctx, worktree, "add", "README.md"); err != nil {
		t.Fatalf("add readme: %v", err)
	}
	if err := runGit(ctx, worktree, "commit", "-m", "Add README"); err != nil {
		t.Fatalf("commit readme: %v", err)
	}
}

func runGit(ctx context.Context, dir string, args ...string) error {
	cmd := exec.CommandContext(ctx, "git", args...)
	cmd.Dir = dir
	if output, err := cmd.CombinedOutput(); err != nil {
		return fmt.Errorf("git %s: %w: %s", strings.Join(args, " "), err, strings.TrimSpace(string(output)))
	}
	return nil
}
