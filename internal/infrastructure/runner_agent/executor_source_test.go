package runneragent_test

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	runneragent "github.com/lyonbrown4d/gity/internal/infrastructure/runner_agent"
)

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
