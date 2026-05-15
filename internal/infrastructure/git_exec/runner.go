package gitexec

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"

	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	"github.com/lyonbrown4d/gity/internal/config"
	"golang.org/x/sys/execabs"
)

var (
	ErrBranchExists            = gitports.ErrBranchExists
	ErrTagExists               = gitports.ErrTagExists
	ErrInvalidBranchName       = gitports.ErrInvalidBranchName
	ErrInvalidTagName          = gitports.ErrInvalidTagName
	ErrSourceReferenceNotFound = gitports.ErrSourceReferenceNotFound
	ErrFileAlreadyExists       = gitports.ErrFileAlreadyExists
	ErrMergeConflict           = gitports.ErrMergeConflict
)

const updateHookScript = `#!/bin/sh
refname="$1"
oldrev="$2"
newrev="$3"
zero="0000000000000000000000000000000000000000"

case "
$GITY_DENY_FORCE_PUSH_REFS
" in
*"
$refname
"*)
	if [ "$oldrev" != "$zero" ] && [ "$newrev" != "$zero" ]; then
		if ! git merge-base --is-ancestor "$oldrev" "$newrev"; then
			echo "force push is not allowed for protected branch: ${refname#refs/heads/}" >&2
			exit 1
		fi
	fi
	;;
esac

exit 0
`

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

func (r *Runner) Run(ctx context.Context, repoPath string, args []string, stdin io.Reader, stdout, stderr io.Writer) error {
	return r.runWithEnv(ctx, repoPath, args, nil, stdin, stdout, stderr)
}

func (r *Runner) RunWithEnv(ctx context.Context, repoPath string, args []string, env map[string]string, stdin io.Reader, stdout, stderr io.Writer) error {
	return r.runWithEnv(ctx, repoPath, args, env, stdin, stdout, stderr)
}

func (r *Runner) runWithEnv(ctx context.Context, repoPath string, args []string, env map[string]string, stdin io.Reader, stdout, stderr io.Writer) error {
	absRepo, err := r.resolveRepoPath(repoPath)
	if err != nil {
		return err
	}
	cmd := execabs.CommandContext(ctx, r.gitBin, args...)
	cmd.Stdin = stdin
	cmd.Stdout = stdout
	cmd.Stderr = stderr
	cmd.Dir = absRepo
	if len(env) > 0 {
		cmd.Env = os.Environ()
		for key, value := range env {
			cmd.Env = append(cmd.Env, key+"="+value)
		}
	}
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("run git %v: %w", args, err)
	}
	return nil
}

func (r *Runner) InitBare(ctx context.Context, repoPath, initialBranch string) error {
	absRepo, err := r.resolveRepoPath(repoPath)
	if err != nil {
		return err
	}
	root, err := filepath.Abs(r.repoRoot)
	if err != nil {
		return fmt.Errorf("resolve repo root: %w", err)
	}
	if err := os.MkdirAll(filepath.Dir(absRepo), 0o750); err != nil {
		return fmt.Errorf("create repo parent dir: %w", err)
	}
	args := []string{"init", "--bare"}
	if strings.TrimSpace(initialBranch) != "" {
		args = append(args, "--initial-branch", strings.TrimSpace(initialBranch))
	}
	args = append(args, absRepo)
	cmd := execabs.CommandContext(ctx, r.gitBin, args...)
	cmd.Dir = root
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("init bare repo %s: %w", repoPath, err)
	}
	return r.EnsureUpdateHook(ctx, repoPath)
}

func (r *Runner) EnsureUpdateHook(_ context.Context, repoPath string) error {
	absRepo, err := r.resolveRepoPath(repoPath)
	if err != nil {
		return err
	}
	hookPath := filepath.Join(absRepo, "hooks", "update")
	if err := os.MkdirAll(filepath.Dir(hookPath), 0o750); err != nil {
		return fmt.Errorf("create git hooks directory: %w", err)
	}
	if err := os.WriteFile(hookPath, []byte(updateHookScript), 0o600); err != nil {
		return fmt.Errorf("write git update hook: %w", err)
	}
	if err := os.Chmod(hookPath, executableHookMode()); err != nil {
		return fmt.Errorf("mark git update hook executable: %w", err)
	}
	return nil
}

func executableHookMode() os.FileMode {
	return 0o755
}

func (r *Runner) CreateBranch(ctx context.Context, repoPath, branchName, sourceRef string) error {
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

func (r *Runner) DeleteBranch(ctx context.Context, repoPath, branchName string) error {
	absRepo, err := r.resolveRepoPath(repoPath)
	if err != nil {
		return err
	}
	branchName = strings.TrimSpace(branchName)
	if err := r.validateBranchName(ctx, absRepo, branchName); err != nil {
		return err
	}
	refName := "refs/heads/" + branchName
	if err := r.runGit(ctx, absRepo, "show-ref", "--verify", "--quiet", refName); err != nil {
		return gitports.ErrReferenceNotFound
	}
	if err := r.runGit(ctx, absRepo, "update-ref", "-d", refName); err != nil {
		return fmt.Errorf("delete branch %s: %w", branchName, err)
	}
	return nil
}

func (r *Runner) CreateTag(ctx context.Context, repoPath, tagName, sourceRef string) error {
	absRepo, err := r.resolveRepoPath(repoPath)
	if err != nil {
		return err
	}
	tagName = strings.TrimSpace(tagName)
	if validationErr := r.validateTagName(ctx, absRepo, tagName); validationErr != nil {
		return validationErr
	}
	refName := "refs/tags/" + tagName
	if existingErr := r.runGit(ctx, absRepo, "show-ref", "--verify", "--quiet", refName); existingErr == nil {
		return ErrTagExists
	}
	sourceRef = strings.TrimSpace(sourceRef)
	if sourceRef == "" {
		sourceRef = "HEAD"
	}
	sha, err := r.runGitOutput(ctx, absRepo, "rev-parse", "--verify", sourceRef+"^{commit}")
	if err != nil {
		return fmt.Errorf("%w: %s", ErrSourceReferenceNotFound, sourceRef)
	}
	if err := r.runGit(ctx, absRepo, "update-ref", refName, strings.TrimSpace(sha)); err != nil {
		return fmt.Errorf("create tag %s: %w", tagName, err)
	}
	return nil
}

func (r *Runner) DeleteTag(ctx context.Context, repoPath, tagName string) error {
	absRepo, err := r.resolveRepoPath(repoPath)
	if err != nil {
		return err
	}
	tagName = strings.TrimSpace(tagName)
	if err := r.validateTagName(ctx, absRepo, tagName); err != nil {
		return err
	}
	refName := "refs/tags/" + tagName
	if err := r.runGit(ctx, absRepo, "show-ref", "--verify", "--quiet", refName); err != nil {
		return gitports.ErrReferenceNotFound
	}
	if err := r.runGit(ctx, absRepo, "update-ref", "-d", refName); err != nil {
		return fmt.Errorf("delete tag %s: %w", tagName, err)
	}
	return nil
}

func (r *Runner) DiffBranches(ctx context.Context, repoPath, targetBranch, sourceBranch string) (string, error) {
	absRepo, err := r.resolveRepoPath(repoPath)
	if err != nil {
		return "", err
	}
	targetBranch = strings.TrimSpace(targetBranch)
	sourceBranch = strings.TrimSpace(sourceBranch)
	if targetErr := r.validateBranchName(ctx, absRepo, targetBranch); targetErr != nil {
		return "", targetErr
	}
	if sourceErr := r.validateBranchName(ctx, absRepo, sourceBranch); sourceErr != nil {
		return "", sourceErr
	}
	output, err := r.runGitOutput(ctx, absRepo, "diff", "--find-renames", "refs/heads/"+targetBranch+"...refs/heads/"+sourceBranch)
	if err != nil {
		return "", fmt.Errorf("%w: diff %s...%s", ErrSourceReferenceNotFound, targetBranch, sourceBranch)
	}
	return output, nil
}

func (r *Runner) Archive(ctx context.Context, repoPath, revision string) ([]byte, error) {
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
	cmd := execabs.CommandContext(ctx, r.gitBin, "archive", "--format=zip", revision)
	cmd.Dir = absRepo
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr
	if err := cmd.Run(); err != nil {
		return nil, fmt.Errorf("%w: archive %s: %w: %s", ErrSourceReferenceNotFound, revision, err, strings.TrimSpace(stderr.String()))
	}
	return stdout.Bytes(), nil
}
