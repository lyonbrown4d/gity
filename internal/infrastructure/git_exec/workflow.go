package gitexec

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	"github.com/samber/oops"
)

type CreateFileCommitInput = gitports.CreateFileCommitInput

func (r *Runner) CreateFileCommit(ctx context.Context, repoPath string, input CreateFileCommitInput) (err error) {
	absRepo, branchName, filePath, message, err := r.prepareFileCommit(ctx, repoPath, input)
	if err != nil {
		return err
	}
	tmpParent, worktree, err := r.cloneToTempWorktree(ctx, "gity-file-commit-*", absRepo)
	if err != nil {
		return err
	}
	defer func() {
		if cleanupErr := os.RemoveAll(tmpParent); cleanupErr != nil && err == nil {
			err = oops.In("git_exec").With("tmp_parent", tmpParent).Wrapf(cleanupErr, "remove temporary file commit worktree")
		}
	}()

	if err := r.checkoutOrCreateBranch(ctx, worktree, branchName); err != nil {
		return err
	}
	if err := r.configureAuthor(ctx, worktree, input.AuthorName, input.AuthorEmail); err != nil {
		return err
	}
	if err := writeNewWorktreeFile(worktree, filePath, input.Content); err != nil {
		return err
	}
	return r.pushFileCommit(ctx, worktree, branchName, filePath, message)
}

func (r *Runner) pushFileCommit(ctx context.Context, worktree, branchName, filePath, message string) error {
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

func (r *Runner) prepareFileCommit(ctx context.Context, repoPath string, input CreateFileCommitInput) (string, string, string, string, error) {
	absRepo, err := r.resolveRepoPath(repoPath)
	if err != nil {
		return "", "", "", "", err
	}
	branchName := strings.TrimSpace(input.BranchName)
	if branchErr := r.validateBranchName(ctx, absRepo, branchName); branchErr != nil {
		return "", "", "", "", branchErr
	}
	filePath, err := normalizeWorktreePath(input.FilePath)
	if err != nil {
		return "", "", "", "", err
	}
	message := strings.TrimSpace(input.Message)
	if message == "" {
		return "", "", "", "", oops.In("git_exec").With("repo_path", repoPath, "branch", branchName, "file_path", filePath).New("commit message is required")
	}
	return absRepo, branchName, filePath, message, nil
}

type MergeBranchesInput = gitports.MergeBranchesInput

func (r *Runner) MergeBranches(ctx context.Context, repoPath string, input MergeBranchesInput) (err error) {
	absRepo, targetBranch, sourceBranch, err := r.prepareMergeBranches(ctx, repoPath, input)
	if err != nil {
		return err
	}
	tmpParent, worktree, err := r.cloneToTempWorktree(ctx, "gity-merge-*", absRepo)
	if err != nil {
		return err
	}
	defer func() {
		if cleanupErr := os.RemoveAll(tmpParent); cleanupErr != nil && err == nil {
			err = oops.In("git_exec").With("tmp_parent", tmpParent).Wrapf(cleanupErr, "remove temporary merge worktree")
		}
	}()

	if err := r.runGit(ctx, worktree, "checkout", targetBranch); err != nil {
		return fmt.Errorf("%w: %s", ErrSourceReferenceNotFound, targetBranch)
	}
	if err := r.runGit(ctx, worktree, "show-ref", "--verify", "--quiet", "refs/remotes/origin/"+sourceBranch); err != nil {
		return fmt.Errorf("%w: %s", ErrSourceReferenceNotFound, sourceBranch)
	}
	if err := r.configureAuthor(ctx, worktree, input.AuthorName, input.AuthorEmail); err != nil {
		return err
	}
	if err := r.mergeBranch(ctx, worktree, sourceBranch, targetBranch, input.Message); err != nil {
		return err
	}
	if err := r.runGit(ctx, worktree, "push", "origin", "HEAD:refs/heads/"+targetBranch); err != nil {
		return fmt.Errorf("push merge commit: %w", err)
	}
	return nil
}

func (r *Runner) prepareMergeBranches(ctx context.Context, repoPath string, input MergeBranchesInput) (string, string, string, error) {
	absRepo, err := r.resolveRepoPath(repoPath)
	if err != nil {
		return "", "", "", err
	}
	targetBranch := strings.TrimSpace(input.TargetBranch)
	sourceBranch := strings.TrimSpace(input.SourceBranch)
	if targetErr := r.validateBranchName(ctx, absRepo, targetBranch); targetErr != nil {
		return "", "", "", targetErr
	}
	if sourceErr := r.validateBranchName(ctx, absRepo, sourceBranch); sourceErr != nil {
		return "", "", "", sourceErr
	}
	return absRepo, targetBranch, sourceBranch, nil
}

func (r *Runner) cloneToTempWorktree(ctx context.Context, pattern, absRepo string) (string, string, error) {
	tmpParent, err := os.MkdirTemp("", pattern)
	if err != nil {
		return "", "", fmt.Errorf("create temporary worktree: %w", err)
	}
	worktree := filepath.Join(tmpParent, "worktree")
	if err := r.runGit(ctx, tmpParent, "clone", absRepo, worktree); err != nil {
		return "", "", fmt.Errorf("clone repository: %w", err)
	}
	return tmpParent, worktree, nil
}

func (r *Runner) checkoutOrCreateBranch(ctx context.Context, worktree, branchName string) error {
	if err := r.runGit(ctx, worktree, "checkout", branchName); err != nil {
		if err := r.runGit(ctx, worktree, "checkout", "-B", branchName); err != nil {
			return fmt.Errorf("checkout branch %s: %w", branchName, err)
		}
	}
	return nil
}

func (r *Runner) configureAuthor(ctx context.Context, worktree, authorName, authorEmail string) error {
	authorName = strings.TrimSpace(authorName)
	if authorName == "" {
		authorName = "Gity"
	}
	authorEmail = strings.TrimSpace(authorEmail)
	if authorEmail == "" {
		authorEmail = "noreply@gity.local"
	}
	if err := r.runGit(ctx, worktree, "config", "user.name", authorName); err != nil {
		return err
	}
	return r.runGit(ctx, worktree, "config", "user.email", authorEmail)
}

func writeNewWorktreeFile(worktree, filePath, content string) error {
	absFile := filepath.Join(worktree, filepath.FromSlash(filePath))
	if _, err := os.Stat(absFile); err == nil {
		return ErrFileAlreadyExists
	} else if !os.IsNotExist(err) {
		return fmt.Errorf("stat target file: %w", err)
	}
	if err := os.MkdirAll(filepath.Dir(absFile), 0o750); err != nil {
		return fmt.Errorf("create target directory: %w", err)
	}
	if err := os.WriteFile(absFile, []byte(content), 0o600); err != nil {
		return fmt.Errorf("write target file: %w", err)
	}
	return nil
}

func (r *Runner) mergeBranch(ctx context.Context, worktree, sourceBranch, targetBranch, message string) error {
	args := []string{"merge", "--no-ff"}
	if message = strings.TrimSpace(message); message != "" {
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
	return nil
}
