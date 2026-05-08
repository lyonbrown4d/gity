package runneragent

import (
	"context"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
)

func checkoutProjectSource(ctx context.Context, cfg Config, job cidomain.ProjectJob, payload ScriptPayload, workDir string, sourceFetcher ScriptSourceFetcher) error {
	repoRoot := strings.TrimSpace(cfg.RepoRoot)
	projectFullPath := strings.TrimSpace(payload.ProjectFullPath)
	if projectFullPath == "" {
		return nil
	}
	if repoRoot == "" {
		if sourceFetcher == nil {
			return nil
		}
		return sourceFetcher(ctx, job, payload, workDir)
	}
	repoPath, err := resolveLocalBareRepo(repoRoot, projectFullPath)
	if err != nil {
		return err
	}
	if _, err := os.Stat(repoPath); err != nil {
		return fmt.Errorf("local repository is not available for checkout: %w", err)
	}
	if _, err := os.Stat(filepath.Join(workDir, ".git")); err == nil {
		if err := runGit(ctx, workDir, "fetch", "--all", "--prune"); err != nil {
			return err
		}
	} else if err := runGit(ctx, workDir, "clone", "--no-checkout", repoPath, "."); err != nil {
		return err
	}

	revision := strings.TrimSpace(payload.CommitSHA)
	if revision != "" {
		return runGit(ctx, workDir, "checkout", "--detach", revision)
	}
	refName := strings.TrimSpace(payload.RefName)
	if refName == "" {
		return nil
	}
	return runGit(ctx, workDir, "checkout", "-B", refName, "origin/"+refName)
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
