package gitexec

import (
	"context"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/DaiYuANg/gity/internal/config"
)

type Runner struct {
	gitBin   string
	repoRoot string
}

func NewRunner(settings config.Settings) *Runner {
	return &Runner{
		gitBin:   settings.Git.Bin,
		repoRoot: settings.Git.RepoRoot,
	}
}

func (r *Runner) Run(ctx context.Context, repoPath string, args []string, stdin io.Reader, stdout io.Writer, stderr io.Writer) error {
	absRepo, err := r.resolveRepoPath(repoPath)
	if err != nil {
		return err
	}
	cmd := exec.CommandContext(ctx, r.gitBin, args...)
	cmd.Stdin = stdin
	cmd.Stdout = stdout
	cmd.Stderr = stderr
	cmd.Dir = absRepo
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("run git %v: %w", args, err)
	}
	return nil
}

func (r *Runner) InitBare(ctx context.Context, repoPath string, initialBranch string) error {
	absRepo, err := r.resolveRepoPath(repoPath)
	if err != nil {
		return err
	}
	root, err := filepath.Abs(r.repoRoot)
	if err != nil {
		return fmt.Errorf("resolve repo root: %w", err)
	}
	if err := os.MkdirAll(filepath.Dir(absRepo), 0o755); err != nil {
		return fmt.Errorf("create repo parent dir: %w", err)
	}
	args := []string{"init", "--bare"}
	if strings.TrimSpace(initialBranch) != "" {
		args = append(args, "--initial-branch", strings.TrimSpace(initialBranch))
	}
	args = append(args, absRepo)
	cmd := exec.CommandContext(ctx, r.gitBin, args...)
	cmd.Dir = root
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("init bare repo %s: %w", repoPath, err)
	}
	return nil
}

func (r *Runner) resolveRepoPath(repoPath string) (string, error) {
	if strings.TrimSpace(repoPath) == "" {
		return "", fmt.Errorf("repo path is required")
	}
	root, err := filepath.Abs(r.repoRoot)
	if err != nil {
		return "", fmt.Errorf("resolve repo root: %w", err)
	}
	absRepo, err := filepath.Abs(filepath.Join(root, repoPath))
	if err != nil {
		return "", fmt.Errorf("resolve repo path: %w", err)
	}
	if absRepo != root && !strings.HasPrefix(absRepo, root+string(filepath.Separator)) {
		return "", fmt.Errorf("repo path escapes repo root")
	}
	return absRepo, nil
}
