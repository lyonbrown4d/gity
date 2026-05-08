package mergerequest

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"

	"github.com/DaiYuANg/gity/internal/infrastructure/git_repo"
)

func mergeRequestCIConfig() string {
	return `
pipeline {
  name = "merge-request"
}

stage test {
  run {
    shell("echo ok")
  }
}
`
}

func findBranch(ctx context.Context, t *testing.T, gitRepository *gitrepo.Service, repoPath, defaultBranch, branchName string) gitrepo.Branch {
	t.Helper()
	branches, err := gitRepository.ListBranches(ctx, repoPath, defaultBranch)
	if err != nil {
		t.Fatalf("list branches: %v", err)
	}
	for _, branch := range branches {
		if branch.Name == branchName {
			return branch
		}
	}
	t.Fatalf("branch %s not found: %+v", branchName, branches)
	return gitrepo.Branch{}
}

func pushFixtureBranches(ctx context.Context, repoRoot, repoPath string) error {
	worktree := filepath.Join(filepath.Dir(repoRoot), "fixture-worktree-mr")
	if err := os.MkdirAll(worktree, 0o750); err != nil {
		return fmt.Errorf("create fixture worktree: %w", err)
	}
	if err := runGit(ctx, worktree, "init", "-b", "main"); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "config", "user.name", "Gity Test"); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "config", "user.email", "test@gity.dev"); err != nil {
		return err
	}
	if err := os.WriteFile(filepath.Join(worktree, "README.md"), []byte("# Hello Gity\n"), 0o600); err != nil {
		return fmt.Errorf("write fixture readme: %w", err)
	}
	if err := runGit(ctx, worktree, "add", "."); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "commit", "-m", "Initial repository content"); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "branch", "feature"); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "checkout", "feature"); err != nil {
		return err
	}
	if err := os.WriteFile(filepath.Join(worktree, "feature.txt"), []byte("feature branch\n"), 0o600); err != nil {
		return fmt.Errorf("write fixture feature file: %w", err)
	}
	if err := runGit(ctx, worktree, "add", "."); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "commit", "-m", "Add feature content"); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "checkout", "main"); err != nil {
		return err
	}

	absRepo := filepath.Join(repoRoot, filepath.FromSlash(repoPath))
	repoURL := "file:///" + filepath.ToSlash(absRepo)
	if err := runGit(ctx, worktree, "push", repoURL, "main:refs/heads/main"); err != nil {
		return err
	}
	return runGit(ctx, worktree, "push", repoURL, "feature:refs/heads/feature")
}

func runGit(ctx context.Context, dir string, args ...string) error {
	cmd := exec.CommandContext(ctx, "git", args...)
	cmd.Dir = dir
	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("git %v: %w: %s", args, err, strings.TrimSpace(string(output)))
	}
	return nil
}
