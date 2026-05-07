package runneragent

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"time"

	"github.com/DaiYuANg/gity/internal/entity"
)

type ScriptPayload struct {
	ProjectFullPath string            `json:"project_full_path"`
	RefName         string            `json:"ref_name"`
	CommitSHA       string            `json:"commit_sha"`
	Script          []string          `json:"script"`
	Env             map[string]string `json:"env"`
	Shell           string            `json:"shell"`
	Artifacts       []string          `json:"artifacts"`
	TimeoutSeconds  int               `json:"timeout_seconds"`
}

type ScriptResult struct {
	ExitCode        int    `json:"exit_code"`
	Output          string `json:"output"`
	OutputTruncated bool   `json:"output_truncated"`
	DurationMillis  int64  `json:"duration_millis"`
	WorkDir         string `json:"work_dir"`
}

type ScriptCancellationChecker func(ctx context.Context) (bool, error)

type ScriptTraceStreamer func(ctx context.Context, output string, outputTruncated bool, durationMillis int64) error

func ExecuteScriptJob(ctx context.Context, cfg Config, job entity.ProjectJob) (string, error) {
	return ExecuteScriptJobWithChecker(ctx, cfg, job, nil)
}

func ExecuteScriptJobWithChecker(ctx context.Context, cfg Config, job entity.ProjectJob, checker ScriptCancellationChecker) (string, error) {
	return ExecuteScriptJobWithTrace(ctx, cfg, job, checker, nil)
}

func ExecuteScriptJobWithTrace(ctx context.Context, cfg Config, job entity.ProjectJob, checker ScriptCancellationChecker, traceStreamer ScriptTraceStreamer) (string, error) {
	var payload ScriptPayload
	if err := json.Unmarshal([]byte(strings.TrimSpace(job.Payload)), &payload); err != nil {
		return "", fmt.Errorf("decode script job payload: %w", err)
	}
	if len(payload.Script) == 0 {
		return "", fmt.Errorf("script job payload requires at least one script line")
	}

	timeout := time.Duration(payload.TimeoutSeconds) * time.Second
	if timeout <= 0 {
		timeout = time.Duration(cfg.LeaseSeconds) * time.Second
	}
	if timeout <= 0 {
		timeout = 10 * time.Minute
	}
	ctx, cancel := context.WithTimeout(ctx, timeout)
	defer cancel()

	cancelRequested := make(chan struct{}, 1)
	done := make(chan struct{})
	defer close(done)

	if checker != nil {
		go func() {
			ticker := time.NewTicker(time.Second)
			defer ticker.Stop()
			for {
				select {
				case <-ctx.Done():
					return
				case <-done:
					return
				case <-ticker.C:
					shouldCancel, err := checker(ctx)
					if err != nil || !shouldCancel {
						continue
					}
					select {
					case cancelRequested <- struct{}{}:
					default:
					}
					cancel()
					return
				}
			}
		}()
	}

	workDir := filepath.Join(cfg.WorkDir, fmt.Sprintf("job-%d-attempt-%d", job.ID, job.Attempts))
	if err := os.MkdirAll(workDir, 0o755); err != nil {
		return "", fmt.Errorf("create job workspace: %w", err)
	}
	if err := checkoutProjectSource(ctx, cfg, payload, workDir); err != nil {
		return "", err
	}

	started := time.Now()
	command := scriptCommand(ctx, payload.Shell, strings.Join(payload.Script, "\n"))
	command.Dir = workDir
	command.Env = append(os.Environ(),
		fmt.Sprintf("GITY_JOB_ID=%d", job.ID),
		fmt.Sprintf("GITY_PROJECT_ID=%d", job.ProjectID),
		fmt.Sprintf("GITY_JOB_ATTEMPT=%d", job.Attempts),
		fmt.Sprintf("GITY_PROJECT_FULL_PATH=%s", payload.ProjectFullPath),
		fmt.Sprintf("GITY_REF_NAME=%s", payload.RefName),
		fmt.Sprintf("GITY_COMMIT_SHA=%s", payload.CommitSHA),
	)
	for key, value := range payload.Env {
		key = strings.TrimSpace(key)
		if key == "" {
			continue
		}
		command.Env = append(command.Env, fmt.Sprintf("%s=%s", key, value))
	}

	output := &cappedBuffer{
		limit:         cfg.MaxOutputBytes,
		ctx:           ctx,
		started:       started,
		traceStreamer: traceStreamer,
	}
	command.Stdout = output
	command.Stderr = output
	err := command.Run()
	exitCode := exitCodeFromError(err)

	canceledByChecker := false
	select {
	case <-cancelRequested:
		canceledByChecker = true
	default:
	}
	if canceledByChecker {
		err = fmt.Errorf("script job canceled")
		exitCode = -1
	} else if ctx.Err() == context.DeadlineExceeded {
		err = fmt.Errorf("script job timed out after %s", timeout)
		exitCode = -1
	}

	result := ScriptResult{
		ExitCode:        exitCode,
		Output:          output.String(),
		OutputTruncated: output.truncated,
		DurationMillis:  time.Since(started).Milliseconds(),
		WorkDir:         workDir,
	}
	encoded, encodeErr := json.Marshal(result)
	if encodeErr != nil {
		return "", fmt.Errorf("encode script result: %w", encodeErr)
	}
	if err != nil {
		return string(encoded), fmt.Errorf("script job failed: exit_code=%d: %w", exitCode, err)
	}
	return string(encoded), nil
}

func scriptCommand(ctx context.Context, shell string, script string) *exec.Cmd {
	normalized := strings.ToLower(strings.TrimSpace(shell))
	if normalized == "" {
		if runtime.GOOS == "windows" {
			return exec.CommandContext(ctx, "powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script)
		}
		return exec.CommandContext(ctx, "/bin/sh", "-lc", script)
	}
	switch normalized {
	case "powershell", "powershell.exe":
		return exec.CommandContext(ctx, "powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script)
	case "pwsh", "pwsh.exe":
		return exec.CommandContext(ctx, "pwsh", "-NoProfile", "-NonInteractive", "-Command", script)
	case "cmd", "cmd.exe":
		return exec.CommandContext(ctx, "cmd.exe", "/C", script)
	case "bash":
		return exec.CommandContext(ctx, "bash", "-lc", script)
	case "sh":
		return exec.CommandContext(ctx, "/bin/sh", "-lc", script)
	default:
		if runtime.GOOS == "windows" {
			return exec.CommandContext(ctx, shell, "/C", script)
		}
		return exec.CommandContext(ctx, shell, "-lc", script)
	}
}

func checkoutProjectSource(ctx context.Context, cfg Config, payload ScriptPayload, workDir string) error {
	repoRoot := strings.TrimSpace(cfg.RepoRoot)
	projectFullPath := strings.TrimSpace(payload.ProjectFullPath)
	if repoRoot == "" || projectFullPath == "" {
		return nil
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

func resolveLocalBareRepo(repoRoot string, projectFullPath string) (string, error) {
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
		return "", fmt.Errorf("runner repository path escapes repo root")
	}
	return repoPath, nil
}

func runGit(ctx context.Context, dir string, args ...string) error {
	command := exec.CommandContext(ctx, "git", args...)
	command.Dir = dir
	output, err := command.CombinedOutput()
	if err == nil {
		return nil
	}
	return fmt.Errorf("git %s: %w: %s", strings.Join(args, " "), err, strings.TrimSpace(string(output)))
}

func exitCodeFromError(err error) int {
	if err == nil {
		return 0
	}
	var exitError *exec.ExitError
	if errors.As(err, &exitError) {
		return exitError.ExitCode()
	}
	return 1
}

type cappedBuffer struct {
	mu            sync.Mutex
	buffer        bytes.Buffer
	limit         int
	truncated     bool
	truncatedSent bool
	ctx           context.Context
	started       time.Time
	traceStreamer ScriptTraceStreamer
}

func (b *cappedBuffer) Write(p []byte) (int, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	if b.limit <= 0 {
		return len(p), nil
	}
	remaining := b.limit - b.buffer.Len()
	if remaining <= 0 {
		b.truncated = true
		if !b.truncatedSent {
			b.truncatedSent = true
			b.stream(nil)
		}
		return len(p), nil
	}
	captured := p
	if len(p) > remaining {
		b.truncated = true
		b.truncatedSent = true
		captured = p[:remaining]
		_, _ = b.buffer.Write(captured)
		b.stream(captured)
		return len(p), nil
	}
	n, err := b.buffer.Write(p)
	b.stream(p)
	return n, err
}

func (b *cappedBuffer) String() string {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buffer.String()
}

func (b *cappedBuffer) stream(chunk []byte) {
	if b.traceStreamer == nil {
		return
	}
	if len(chunk) == 0 && !b.truncated {
		return
	}
	duration := int64(0)
	if !b.started.IsZero() {
		duration = time.Since(b.started).Milliseconds()
	}
	_ = b.traceStreamer(b.ctx, string(chunk), b.truncated, duration)
}
