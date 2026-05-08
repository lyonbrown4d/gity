package runneragent

import (
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

	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	"golang.org/x/sys/execabs"
)

func ExecuteScriptJob(ctx context.Context, cfg Config, job cidomain.ProjectJob) (string, error) {
	return ExecuteScriptJobWithChecker(ctx, cfg, job, nil)
}

func ExecuteScriptJobWithChecker(ctx context.Context, cfg Config, job cidomain.ProjectJob, checker ScriptCancellationChecker) (string, error) {
	return ExecuteScriptJobWithTrace(ctx, cfg, job, checker, nil)
}

func ExecuteScriptJobWithTrace(ctx context.Context, cfg Config, job cidomain.ProjectJob, checker ScriptCancellationChecker, traceStreamer ScriptTraceStreamer) (string, error) {
	return ExecuteScriptJobWithSource(ctx, cfg, job, checker, traceStreamer, nil)
}

func ExecuteScriptJobWithSource(ctx context.Context, cfg Config, job cidomain.ProjectJob, checker ScriptCancellationChecker, traceStreamer ScriptTraceStreamer, sourceFetcher ScriptSourceFetcher) (string, error) {
	var payload ScriptPayload
	if err := json.Unmarshal([]byte(strings.TrimSpace(job.Payload)), &payload); err != nil {
		return "", fmt.Errorf("decode script job payload: %w", err)
	}
	if len(payload.Script) == 0 {
		return "", errors.New("script job payload requires at least one script line")
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
	if err := os.MkdirAll(workDir, 0o750); err != nil {
		return "", fmt.Errorf("create job workspace: %w", err)
	}
	if err := checkoutProjectSource(ctx, cfg, job, payload, workDir, sourceFetcher); err != nil {
		return "", err
	}

	started := time.Now()
	command := scriptCommand(ctx, payload.Shell, strings.Join(payload.Script, "\n"))
	command.Dir = workDir
	command.Env = append(os.Environ(),
		fmt.Sprintf("GITY_JOB_ID=%d", job.ID),
		fmt.Sprintf("GITY_PROJECT_ID=%d", job.ProjectID),
		fmt.Sprintf("GITY_JOB_ATTEMPT=%d", job.Attempts),
		"GITY_PROJECT_FULL_PATH="+payload.ProjectFullPath,
		"GITY_REF_NAME="+payload.RefName,
		"GITY_COMMIT_SHA="+payload.CommitSHA,
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
		err = errors.New("script job canceled")
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

func scriptCommand(ctx context.Context, shell, script string) *exec.Cmd {
	normalized := strings.ToLower(strings.TrimSpace(shell))
	if normalized == "" {
		if runtime.GOOS == "windows" {
			return execabs.CommandContext(ctx, "powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script)
		}
		return execabs.CommandContext(ctx, "/bin/sh", "-lc", script)
	}
	switch normalized {
	case "powershell", "powershell.exe":
		return execabs.CommandContext(ctx, "powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script)
	case "pwsh", "pwsh.exe":
		return execabs.CommandContext(ctx, "pwsh", "-NoProfile", "-NonInteractive", "-Command", script)
	case "cmd", "cmd.exe":
		return execabs.CommandContext(ctx, "cmd.exe", "/C", script)
	case "bash":
		return execabs.CommandContext(ctx, "bash", "-lc", script)
	case "sh":
		return execabs.CommandContext(ctx, "/bin/sh", "-lc", script)
	default:
		if runtime.GOOS == "windows" {
			return execabs.CommandContext(ctx, shell, "/C", script)
		}
		return execabs.CommandContext(ctx, shell, "-lc", script)
	}
}

func runGit(ctx context.Context, dir string, args ...string) error {
	command := execabs.CommandContext(ctx, "git", args...)
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
