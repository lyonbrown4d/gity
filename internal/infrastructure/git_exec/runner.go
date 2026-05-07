package gitexec

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	"github.com/DaiYuANg/gity/internal/config"
)

var (
	ErrBranchExists            = gitports.ErrBranchExists
	ErrInvalidBranchName       = gitports.ErrInvalidBranchName
	ErrSourceReferenceNotFound = gitports.ErrSourceReferenceNotFound
	ErrFileAlreadyExists       = gitports.ErrFileAlreadyExists
	ErrMergeConflict           = gitports.ErrMergeConflict
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

func (r *Runner) CreateBranch(ctx context.Context, repoPath string, branchName string, sourceRef string) error {
	absRepo, err := r.resolveRepoPath(repoPath)
	if err != nil {
		return err
	}
	branchName = strings.TrimSpace(branchName)
	if err := r.validateBranchName(ctx, absRepo, branchName); err != nil {
		return err
	}
	if err := r.runGit(ctx, absRepo, "show-ref", "--verify", "--quiet", "refs/heads/"+branchName); err == nil {
		return ErrBranchExists
	}
	sourceRef = strings.TrimSpace(sourceRef)
	if sourceRef == "" {
		sourceRef = "HEAD"
	}
	if err := r.runGit(ctx, absRepo, "branch", branchName, sourceRef); err != nil {
		return fmt.Errorf("%w: %s", ErrSourceReferenceNotFound, sourceRef)
	}
	return nil
}

type CreateFileCommitInput = gitports.CreateFileCommitInput

func (r *Runner) CreateFileCommit(ctx context.Context, repoPath string, input CreateFileCommitInput) error {
	absRepo, err := r.resolveRepoPath(repoPath)
	if err != nil {
		return err
	}
	branchName := strings.TrimSpace(input.BranchName)
	if err := r.validateBranchName(ctx, absRepo, branchName); err != nil {
		return err
	}
	filePath, err := normalizeWorktreePath(input.FilePath)
	if err != nil {
		return err
	}
	message := strings.TrimSpace(input.Message)
	if message == "" {
		return fmt.Errorf("commit message is required")
	}

	tmpParent, err := os.MkdirTemp("", "gity-file-commit-*")
	if err != nil {
		return fmt.Errorf("create temporary worktree: %w", err)
	}
	defer os.RemoveAll(tmpParent)

	worktree := filepath.Join(tmpParent, "worktree")
	if err := r.runGit(ctx, tmpParent, "clone", absRepo, worktree); err != nil {
		return fmt.Errorf("clone repository: %w", err)
	}
	if err := r.runGit(ctx, worktree, "checkout", branchName); err != nil {
		if err := r.runGit(ctx, worktree, "checkout", "-B", branchName); err != nil {
			return fmt.Errorf("checkout branch %s: %w", branchName, err)
		}
	}
	authorName := strings.TrimSpace(input.AuthorName)
	if authorName == "" {
		authorName = "Gity"
	}
	authorEmail := strings.TrimSpace(input.AuthorEmail)
	if authorEmail == "" {
		authorEmail = "noreply@gity.local"
	}
	if err := r.runGit(ctx, worktree, "config", "user.name", authorName); err != nil {
		return err
	}
	if err := r.runGit(ctx, worktree, "config", "user.email", authorEmail); err != nil {
		return err
	}
	absFile := filepath.Join(worktree, filepath.FromSlash(filePath))
	if _, err := os.Stat(absFile); err == nil {
		return ErrFileAlreadyExists
	} else if err != nil && !os.IsNotExist(err) {
		return fmt.Errorf("stat target file: %w", err)
	}
	if err := os.MkdirAll(filepath.Dir(absFile), 0o755); err != nil {
		return fmt.Errorf("create target directory: %w", err)
	}
	if err := os.WriteFile(absFile, []byte(input.Content), 0o644); err != nil {
		return fmt.Errorf("write target file: %w", err)
	}
	if err := r.runGit(ctx, worktree, "add", "--", filePath); err != nil {
		return err
	}
	if err := r.runGit(ctx, worktree, "commit", "-m", message); err != nil {
		return fmt.Errorf("commit file: %w", err)
	}
	if err := r.runGit(ctx, worktree, "push", "origin", "HEAD:refs/heads/"+branchName); err != nil {
		return fmt.Errorf("push file commit: %w", err)
	}
	return nil
}

func (r *Runner) DiffBranches(ctx context.Context, repoPath string, targetBranch string, sourceBranch string) (string, error) {
	absRepo, err := r.resolveRepoPath(repoPath)
	if err != nil {
		return "", err
	}
	targetBranch = strings.TrimSpace(targetBranch)
	sourceBranch = strings.TrimSpace(sourceBranch)
	if err := r.validateBranchName(ctx, absRepo, targetBranch); err != nil {
		return "", err
	}
	if err := r.validateBranchName(ctx, absRepo, sourceBranch); err != nil {
		return "", err
	}
	output, err := r.runGitOutput(ctx, absRepo, "diff", "--find-renames", "refs/heads/"+targetBranch+"...refs/heads/"+sourceBranch)
	if err != nil {
		return "", fmt.Errorf("%w: diff %s...%s", ErrSourceReferenceNotFound, targetBranch, sourceBranch)
	}
	return output, nil
}

func (r *Runner) Archive(ctx context.Context, repoPath string, revision string) ([]byte, error) {
	absRepo, err := r.resolveRepoPath(repoPath)
	if err != nil {
		return nil, err
	}
	revision = strings.TrimSpace(revision)
	if revision == "" || strings.HasPrefix(revision, "-") {
		return nil, ErrSourceReferenceNotFound
	}
	var stdout bytes.Buffer
	var stderr bytes.Buffer
	cmd := exec.CommandContext(ctx, r.gitBin, "archive", "--format=zip", revision)
	cmd.Dir = absRepo
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr
	if err := cmd.Run(); err != nil {
		return nil, fmt.Errorf("%w: archive %s: %v: %s", ErrSourceReferenceNotFound, revision, err, strings.TrimSpace(stderr.String()))
	}
	return stdout.Bytes(), nil
}

type MergeBranchesInput = gitports.MergeBranchesInput

func (r *Runner) MergeBranches(ctx context.Context, repoPath string, input MergeBranchesInput) error {
	absRepo, err := r.resolveRepoPath(repoPath)
	if err != nil {
		return err
	}
	targetBranch := strings.TrimSpace(input.TargetBranch)
	sourceBranch := strings.TrimSpace(input.SourceBranch)
	if err := r.validateBranchName(ctx, absRepo, targetBranch); err != nil {
		return err
	}
	if err := r.validateBranchName(ctx, absRepo, sourceBranch); err != nil {
		return err
	}

	tmpParent, err := os.MkdirTemp("", "gity-merge-*")
	if err != nil {
		return fmt.Errorf("create temporary merge worktree: %w", err)
	}
	defer os.RemoveAll(tmpParent)

	worktree := filepath.Join(tmpParent, "worktree")
	if err := r.runGit(ctx, tmpParent, "clone", absRepo, worktree); err != nil {
		return fmt.Errorf("clone repository: %w", err)
	}
	if err := r.runGit(ctx, worktree, "checkout", targetBranch); err != nil {
		return fmt.Errorf("%w: %s", ErrSourceReferenceNotFound, targetBranch)
	}
	if err := r.runGit(ctx, worktree, "show-ref", "--verify", "--quiet", "refs/remotes/origin/"+sourceBranch); err != nil {
		return fmt.Errorf("%w: %s", ErrSourceReferenceNotFound, sourceBranch)
	}
	authorName := strings.TrimSpace(input.AuthorName)
	if authorName == "" {
		authorName = "Gity"
	}
	authorEmail := strings.TrimSpace(input.AuthorEmail)
	if authorEmail == "" {
		authorEmail = "noreply@gity.local"
	}
	if err := r.runGit(ctx, worktree, "config", "user.name", authorName); err != nil {
		return err
	}
	if err := r.runGit(ctx, worktree, "config", "user.email", authorEmail); err != nil {
		return err
	}
	message := strings.TrimSpace(input.Message)
	args := []string{"merge", "--no-ff"}
	if message != "" {
		args = append(args, "-m", message)
	} else {
		args = append(args, "--no-edit")
	}
	args = append(args, "origin/"+sourceBranch)
	if output, err := r.runGitOutput(ctx, worktree, args...); err != nil {
		if isMergeConflictOutput(output) {
			return fmt.Errorf("%w: %s", ErrMergeConflict, strings.TrimSpace(output))
		}
		return fmt.Errorf("merge branch %s into %s: %w: %s", sourceBranch, targetBranch, err, strings.TrimSpace(output))
	}
	if err := r.runGit(ctx, worktree, "push", "origin", "HEAD:refs/heads/"+targetBranch); err != nil {
		return fmt.Errorf("push merge commit: %w", err)
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

func (r *Runner) validateBranchName(ctx context.Context, dir string, branchName string) error {
	if branchName == "" || strings.HasPrefix(branchName, "-") || strings.Contains(branchName, "..") {
		return ErrInvalidBranchName
	}
	if err := r.runGit(ctx, dir, "check-ref-format", "--branch", branchName); err != nil {
		return ErrInvalidBranchName
	}
	return nil
}

func (r *Runner) runGit(ctx context.Context, dir string, args ...string) error {
	_, err := r.runGitOutput(ctx, dir, args...)
	return err
}

func (r *Runner) runGitOutput(ctx context.Context, dir string, args ...string) (string, error) {
	cmd := exec.CommandContext(ctx, r.gitBin, args...)
	cmd.Dir = dir
	output, err := cmd.CombinedOutput()
	if err != nil {
		return string(output), fmt.Errorf("git %v: %w: %s", args, err, strings.TrimSpace(string(output)))
	}
	return string(output), nil
}

func normalizeWorktreePath(value string) (string, error) {
	normalized := strings.Trim(strings.ReplaceAll(value, "\\", "/"), "/")
	if normalized == "" || strings.HasPrefix(normalized, "../") || strings.Contains(normalized, "/../") || normalized == ".." {
		return "", fmt.Errorf("invalid file path")
	}
	if filepath.IsAbs(normalized) {
		return "", fmt.Errorf("invalid file path")
	}
	return normalized, nil
}

func isMergeConflictOutput(output string) bool {
	output = strings.ToLower(output)
	return strings.Contains(output, "conflict") || strings.Contains(output, "automatic merge failed")
}
