package gitexec

import (
	"context"
	"fmt"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/samber/oops"
	"golang.org/x/sys/execabs"
)

func (r *Runner) resolveRepoPath(repoPath string) (string, error) {
	if strings.TrimSpace(repoPath) == "" {
		return "", oops.In("git_exec").New("repo path is required")
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
		return "", oops.In("git_exec").With("repo_path", repoPath, "root", root, "abs_repo", absRepo).New("repo path escapes repo root")
	}
	return absRepo, nil
}

func (r *Runner) validateBranchName(ctx context.Context, dir, branchName string) error {
	if branchName == "" || strings.HasPrefix(branchName, "-") || strings.Contains(branchName, "..") {
		return ErrInvalidBranchName
	}
	if err := r.runGit(ctx, dir, "check-ref-format", "--branch", branchName); err != nil {
		return ErrInvalidBranchName
	}
	return nil
}

func (r *Runner) validateTagName(ctx context.Context, dir, tagName string) error {
	if tagName == "" || strings.HasPrefix(tagName, "-") || strings.Contains(tagName, "..") {
		return ErrInvalidTagName
	}
	if err := r.runGit(ctx, dir, "check-ref-format", "refs/tags/"+tagName); err != nil {
		return ErrInvalidTagName
	}
	return nil
}

func (r *Runner) runGit(ctx context.Context, dir string, args ...string) error {
	_, err := r.runGitOutput(ctx, dir, args...)
	return err
}

func (r *Runner) runGitOutput(ctx context.Context, dir string, args ...string) (string, error) {
	cmd := r.gitCommand(ctx, args...)
	cmd.Dir = dir
	output, err := cmd.CombinedOutput()
	if err != nil {
		return string(output), fmt.Errorf("git %v: %w: %s", args, err, strings.TrimSpace(string(output)))
	}
	return string(output), nil
}

func (r *Runner) gitCommand(ctx context.Context, args ...string) *exec.Cmd {
	var command *exec.Cmd
	if r.usesGitExe() {
		command = execabs.CommandContext(ctx, "git.exe")
	} else {
		command = execabs.CommandContext(ctx, "git")
	}
	command.Args = append(command.Args, args...)
	return command
}

func (r *Runner) usesGitExe() bool {
	return strings.EqualFold(filepath.Base(strings.TrimSpace(r.gitBin)), "git.exe")
}

func normalizeWorktreePath(value string) (string, error) {
	normalized := strings.Trim(strings.ReplaceAll(value, "\\", "/"), "/")
	if normalized == "" || strings.HasPrefix(normalized, "../") || strings.Contains(normalized, "/../") || normalized == ".." {
		return "", oops.In("git_exec").With("file_path", value, "normalized", normalized).New("invalid file path")
	}
	if filepath.IsAbs(normalized) {
		return "", oops.In("git_exec").With("file_path", value, "normalized", normalized).New("invalid file path")
	}
	return normalized, nil
}

func isMergeConflictOutput(output string) bool {
	output = strings.ToLower(output)
	return strings.Contains(output, "conflict") || strings.Contains(output, "automatic merge failed")
}
