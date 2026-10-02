package gitexec

import (
	"context"
	_ "embed"
	"errors"
	"fmt"
	"os"
	"path/filepath"
)

//go:embed update_hook.sh
var updateHookScript string

func (r *Runner) EnsureUpdateHook(_ context.Context, repoPath string) error {
	absRepo, resolveErr := r.resolveRepoPath(repoPath)
	if resolveErr != nil {
		return resolveErr
	}
	hookPath := filepath.Join(absRepo, "hooks", "update")
	if mkdirErr := os.MkdirAll(filepath.Dir(hookPath), 0o750); mkdirErr != nil {
		return fmt.Errorf("create git hooks directory: %w", mkdirErr)
	}
	current, currentErr := currentUpdateHook(absRepo)
	if currentErr != nil {
		return currentErr
	}
	if current {
		if chmodErr := os.Chmod(hookPath, executableHookMode()); chmodErr != nil {
			return fmt.Errorf("mark git update hook executable: %w", chmodErr)
		}
		return nil
	}
	return installUpdateHook(hookPath)
}

func currentUpdateHook(repoPath string) (bool, error) {
	repoRoot, openErr := os.OpenRoot(repoPath)
	if openErr != nil {
		return false, fmt.Errorf("open git repository root: %w", openErr)
	}
	existing, readErr := repoRoot.ReadFile(filepath.Join("hooks", "update"))
	if closeErr := repoRoot.Close(); closeErr != nil {
		return false, fmt.Errorf("close git repository root: %w", closeErr)
	}
	return readErr == nil && string(existing) == updateHookScript, nil
}

func installUpdateHook(hookPath string) (returnErr error) {
	tmpHook, err := os.CreateTemp(filepath.Dir(hookPath), "update-*")
	if err != nil {
		return fmt.Errorf("create git update hook temp file: %w", err)
	}
	tmpPath := tmpHook.Name()
	defer func() {
		returnErr = errors.Join(returnErr, removeTemporaryHook(tmpPath))
	}()
	_, writeErr := tmpHook.WriteString(updateHookScript)
	if finalizeErr := finalizeTemporaryHook(tmpHook, writeErr); finalizeErr != nil {
		return finalizeErr
	}
	if chmodErr := os.Chmod(tmpPath, executableHookMode()); chmodErr != nil {
		return fmt.Errorf("mark git update hook temp file executable: %w", chmodErr)
	}
	if renameErr := os.Rename(tmpPath, hookPath); renameErr != nil {
		return fmt.Errorf("install git update hook: %w", renameErr)
	}
	return nil
}

func finalizeTemporaryHook(tmpHook *os.File, writeErr error) error {
	closeErr := tmpHook.Close()
	if writeErr != nil {
		return errors.Join(
			fmt.Errorf("write git update hook temp file: %w", writeErr),
			wrapTemporaryHookCloseError(closeErr),
		)
	}
	return wrapTemporaryHookCloseError(closeErr)
}

func wrapTemporaryHookCloseError(err error) error {
	if err == nil {
		return nil
	}
	return fmt.Errorf("close git update hook temp file: %w", err)
}

func removeTemporaryHook(path string) error {
	removeErr := os.Remove(path)
	if removeErr == nil || errors.Is(removeErr, os.ErrNotExist) {
		return nil
	}
	return fmt.Errorf("remove git update hook temp file: %w", removeErr)
}

func executableHookMode() os.FileMode {
	return 0o755
}
