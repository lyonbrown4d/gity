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
	"time"

	"github.com/DaiYuANg/gity/internal/entity"
)

type ScriptPayload struct {
	Script         []string          `json:"script"`
	Env            map[string]string `json:"env"`
	Shell          string            `json:"shell"`
	Artifacts      []string          `json:"artifacts"`
	TimeoutSeconds int               `json:"timeout_seconds"`
}

type ScriptResult struct {
	ExitCode        int    `json:"exit_code"`
	Output          string `json:"output"`
	OutputTruncated bool   `json:"output_truncated"`
	DurationMillis  int64  `json:"duration_millis"`
	WorkDir         string `json:"work_dir"`
}

type ScriptCancellationChecker func(ctx context.Context) (bool, error)

func ExecuteScriptJob(ctx context.Context, cfg Config, job entity.ProjectJob) (string, error) {
	return ExecuteScriptJobWithChecker(ctx, cfg, job, nil)
}

func ExecuteScriptJobWithChecker(ctx context.Context, cfg Config, job entity.ProjectJob, checker ScriptCancellationChecker) (string, error) {
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

	started := time.Now()
	command := scriptCommand(ctx, payload.Shell, strings.Join(payload.Script, "\n"))
	command.Dir = workDir
	command.Env = append(os.Environ(),
		fmt.Sprintf("GITY_JOB_ID=%d", job.ID),
		fmt.Sprintf("GITY_PROJECT_ID=%d", job.ProjectID),
		fmt.Sprintf("GITY_JOB_ATTEMPT=%d", job.Attempts),
	)
	for key, value := range payload.Env {
		key = strings.TrimSpace(key)
		if key == "" {
			continue
		}
		command.Env = append(command.Env, fmt.Sprintf("%s=%s", key, value))
	}

	output := &cappedBuffer{limit: cfg.MaxOutputBytes}
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
	bytes.Buffer
	limit     int
	truncated bool
}

func (b *cappedBuffer) Write(p []byte) (int, error) {
	if b.limit <= 0 {
		return len(p), nil
	}
	remaining := b.limit - b.Buffer.Len()
	if remaining <= 0 {
		b.truncated = true
		return len(p), nil
	}
	if len(p) > remaining {
		b.truncated = true
		_, _ = b.Buffer.Write(p[:remaining])
		return len(p), nil
	}
	return b.Buffer.Write(p)
}
