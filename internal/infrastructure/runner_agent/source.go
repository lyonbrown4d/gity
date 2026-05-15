package runneragent

import (
	"context"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	"golang.org/x/sys/execabs"
)

func checkoutProjectSource(ctx context.Context, cfg Config, job cidomain.ProjectJob, payload ScriptPayload, workDir string, sourceFetcher ScriptSourceFetcher) error {
	repoRoot := strings.TrimSpace(cfg.RepoRoot)
	projectFullPath := strings.TrimSpace(payload.ProjectFullPath)
	if projectFullPath == "" {
		return nil
	}
	if repoRoot == "" {
		return fetchProjectSource(ctx, job, payload, workDir, sourceFetcher)
	}
	repoPath, err := resolveLocalBareRepo(repoRoot, projectFullPath)
	if err != nil {
		return err
	}
	if _, err := os.Stat(repoPath); err != nil {
		return fmt.Errorf("local repository is not available for checkout: %w", err)
	}
	if err := prepareLocalCheckout(ctx, repoPath, workDir); err != nil {
		return err
	}
	return checkoutSourceRevision(ctx, workDir, payload)
}

func fetchProjectSource(ctx context.Context, job cidomain.ProjectJob, payload ScriptPayload, workDir string, sourceFetcher ScriptSourceFetcher) error {
	if sourceFetcher == nil {
		return nil
	}
	return sourceFetcher(ctx, job, payload, workDir)
}

func prepareLocalCheckout(ctx context.Context, repoPath, workDir string) error {
	if _, err := os.Stat(filepath.Join(workDir, ".git")); err == nil {
		if err := gitFetchAllPrune(ctx, workDir); err != nil {
			return err
		}
	} else if err := gitCloneNoCheckout(ctx, workDir, repoPath); err != nil {
		return err
	}
	return nil
}

func checkoutSourceRevision(ctx context.Context, workDir string, payload ScriptPayload) error {
	revision := strings.TrimSpace(payload.CommitSHA)
	if revision != "" {
		return gitCheckoutDetach(ctx, workDir, revision)
	}
	refName := strings.TrimSpace(payload.RefName)
	if refName == "" {
		return nil
	}
	return gitCheckoutBranch(ctx, workDir, refName)
}

func gitFetchAllPrune(ctx context.Context, workDir string) error {
	command := gitCommand(ctx, "fetch", "--all", "--prune")
	return runGitCommand(command, workDir, "fetch --all --prune")
}

func gitCloneNoCheckout(ctx context.Context, workDir, repoPath string) error {
	command := gitCommand(ctx, "clone", "--no-checkout", repoPath, ".")
	return runGitCommand(command, workDir, "clone --no-checkout")
}

func gitCheckoutDetach(ctx context.Context, workDir, revision string) error {
	command := gitCommand(ctx, "checkout", "--detach", revision)
	return runGitCommand(command, workDir, "checkout --detach")
}

func gitCheckoutBranch(ctx context.Context, workDir, refName string) error {
	command := gitCommand(ctx, "checkout", "-B", refName, "origin/"+refName)
	return runGitCommand(command, workDir, "checkout branch")
}

func gitCommand(ctx context.Context, args ...string) *exec.Cmd {
	command := execabs.CommandContext(ctx, "git")
	command.Args = append(command.Args, args...)
	return command
}

func runGitCommand(command *exec.Cmd, dir, operation string) error {
	command.Dir = dir
	output, err := command.CombinedOutput()
	if err == nil {
		return nil
	}
	return fmt.Errorf("git %s: %w: %s", operation, err, strings.TrimSpace(string(output)))
}

func resolveLocalBareRepo(repoRoot, projectFullPath string) (string, error) {
	root, err := filepath.Abs(repoRoot)
	if err != nil {
		return "", fmt.Errorf("resolve runner repo root: %w", err)
	}
	normalized := strings.Trim(strings.ReplaceAll(projectFullPath, "\\", "/"), "/")
	if normalized == "" || strings.Contains(normalized, "..") || filepath.IsAbs(normalized) {
		return "", fmt.Errorf("invalid project full path for checkout: %s", projectFullPath)
	}
	repoPath, err := filepath.Abs(filepath.Join(root, filepath.FromSlash(normalized)+".git"))
	if err != nil {
		return "", fmt.Errorf("resolve runner repository path: %w", err)
	}
	if repoPath != root && !strings.HasPrefix(repoPath, root+string(filepath.Separator)) {
		return "", errors.New("runner repository path escapes repo root")
	}
	return repoPath, nil
}
