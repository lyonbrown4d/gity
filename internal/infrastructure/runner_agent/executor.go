package runneragent

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
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
	payload, err := decodeScriptPayload(job)
	if err != nil {
		return "", err
	}
	if validationErr := validateScriptPayload(cfg, payload); validationErr != nil {
		return "", validationErr
	}
	timeout := scriptTimeout(cfg, payload)
	ctx, cancel := context.WithTimeout(ctx, timeout)
	defer cancel()

	cancelRequested, stopWatcher := startScriptCancellationWatcher(ctx, cancel, checker)
	defer stopWatcher()

	workDir, err := createScriptWorkspace(cfg, job)
	if err != nil {
		return "", err
	}
	if checkoutErr := checkoutProjectSource(ctx, cfg, job, payload, workDir, sourceFetcher); checkoutErr != nil {
		return "", checkoutErr
	}

	started := time.Now()
	output := newCappedBuffer(ctx, started, traceStreamer, cfg, payload)
	runner, err := resolveScriptRunner(cfg, payload)
	if err != nil {
		return "", err
	}
	err = runner.run(ctx, cfg, job, payload, workDir, output)
	result, resultErr := encodeScriptResult(started, workDir, output, resolveScriptError(ctx, err, timeout, cancelRequested))
	if cleanupErr := cleanupScriptWorkspace(cfg, workDir); cleanupErr != nil {
		if resultErr != nil {
			return result, errors.Join(resultErr, cleanupErr)
		}
		return result, cleanupErr
	}
	return result, resultErr
}

func decodeScriptPayload(job cidomain.ProjectJob) (ScriptPayload, error) {
	var payload ScriptPayload
	if err := json.Unmarshal([]byte(strings.TrimSpace(job.Payload)), &payload); err != nil {
		return ScriptPayload{}, fmt.Errorf("decode script job payload: %w", err)
	}
	if len(payload.Script) == 0 {
		return ScriptPayload{}, errors.New("script job payload requires at least one script line")
	}
	return payload, nil
}

func scriptTimeout(cfg Config, payload ScriptPayload) time.Duration {
	timeout := time.Duration(payload.TimeoutSeconds) * time.Second
	if timeout > 0 {
		return timeout
	}
	timeout = time.Duration(cfg.LeaseSeconds) * time.Second
	if timeout > 0 {
		return timeout
	}
	return 10 * time.Minute
}

func validateScriptPayload(cfg Config, payload ScriptPayload) error {
	if !shellAllowed(cfg, payload.Shell) {
		return fmt.Errorf("script shell %q is not allowed", strings.TrimSpace(payload.Shell))
	}
	return nil
}

func shellAllowed(cfg Config, shell string) bool {
	normalized := normalizeShellName(shell)
	if normalized == "" {
		return true
	}
	if !supportedScriptShell(normalized) {
		return false
	}
	allowed := cfg.AllowedShells
	if len(allowed) == 0 {
		allowed = defaultAllowedShells()
	}
	for _, item := range allowed {
		if normalizeShellName(item) == normalized {
			return true
		}
	}
	return false
}

func normalizeShellName(value string) string {
	normalized := strings.TrimSpace(strings.ToLower(value))
	normalized = strings.TrimSuffix(normalized, ".exe")
	return normalized
}

func startScriptCancellationWatcher(ctx context.Context, cancel context.CancelFunc, checker ScriptCancellationChecker) (chan struct{}, func()) {
	cancelRequested := make(chan struct{}, 1)
	done := make(chan struct{})
	if checker != nil {
		go watchScriptCancellation(ctx, cancel, checker, cancelRequested, done)
	}
	return cancelRequested, func() { close(done) }
}

func watchScriptCancellation(ctx context.Context, cancel context.CancelFunc, checker ScriptCancellationChecker, cancelRequested chan<- struct{}, done <-chan struct{}) {
	ticker := time.NewTicker(time.Second)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-done:
			return
		case <-ticker.C:
			if shouldCancelScript(ctx, checker) {
				requestScriptCancellation(cancelRequested, cancel)
				return
			}
		}
	}
}

func shouldCancelScript(ctx context.Context, checker ScriptCancellationChecker) bool {
	shouldCancel, err := checker(ctx)
	return err == nil && shouldCancel
}

func requestScriptCancellation(cancelRequested chan<- struct{}, cancel context.CancelFunc) {
	select {
	case cancelRequested <- struct{}{}:
	default:
	}
	cancel()
}

func createScriptWorkspace(cfg Config, job cidomain.ProjectJob) (string, error) {
	workDir := filepath.Join(cfg.WorkDir, fmt.Sprintf("job-%d-attempt-%d", job.ID, job.Attempts))
	if err := os.MkdirAll(workDir, 0o750); err != nil {
		return "", fmt.Errorf("create job workspace: %w", err)
	}
	return workDir, nil
}

func cleanupScriptWorkspace(cfg Config, workDir string) error {
	if !cfg.CleanWorkspace {
		return nil
	}
	root, err := filepath.Abs(cfg.WorkDir)
	if err != nil {
		return fmt.Errorf("resolve runner workspace root: %w", err)
	}
	target, err := filepath.Abs(workDir)
	if err != nil {
		return fmt.Errorf("resolve job workspace: %w", err)
	}
	if target == root || !strings.HasPrefix(target, root+string(os.PathSeparator)) {
		return fmt.Errorf("refuse to cleanup workspace outside runner root: %s", target)
	}
	if err := os.RemoveAll(target); err != nil {
		return fmt.Errorf("cleanup job workspace: %w", err)
	}
	return nil
}

func newCappedBuffer(ctx context.Context, started time.Time, traceStreamer ScriptTraceStreamer, cfg Config, payload ScriptPayload) *cappedBuffer {
	return &cappedBuffer{
		limit:         cfg.MaxOutputBytes,
		ctx:           ctx,
		started:       started,
		traceStreamer: traceStreamer,
		maskedValues:  payload.MaskedValues,
	}
}

func scriptEnvironment(job cidomain.ProjectJob, payload ScriptPayload) []string {
	env := append(os.Environ(),
		fmt.Sprintf("GITY_JOB_ID=%d", job.ID),
		fmt.Sprintf("GITY_PROJECT_ID=%d", job.ProjectID),
		fmt.Sprintf("GITY_JOB_ATTEMPT=%d", job.Attempts),
		"GITY_PROJECT_FULL_PATH="+payload.ProjectFullPath,
		"GITY_REF_NAME="+payload.RefName,
		"GITY_COMMIT_SHA="+payload.CommitSHA,
	)
	for key, value := range payload.Env {
		key = strings.TrimSpace(key)
		if key != "" {
			env = append(env, fmt.Sprintf("%s=%s", key, value))
		}
	}
	return env
}

func resolveScriptError(ctx context.Context, err error, timeout time.Duration, cancelRequested <-chan struct{}) error {
	select {
	case <-cancelRequested:
		return errors.New("script job canceled")
	default:
	}
	if ctx.Err() == context.DeadlineExceeded {
		return fmt.Errorf("script job timed out after %s", timeout)
	}
	return err
}

func encodeScriptResult(started time.Time, workDir string, output *cappedBuffer, err error) (string, error) {
	exitCode := exitCodeFromError(err)
	if isForcedScriptExit(err) {
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

func isForcedScriptExit(err error) bool {
	if err == nil {
		return false
	}
	message := err.Error()
	return strings.Contains(message, "script job timed out") || strings.Contains(message, "script job canceled")
}
