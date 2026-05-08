package runneragent

import (
	"archive/zip"
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"time"

	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	"github.com/samber/oops"
	"golang.org/x/sys/execabs"
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

type ScriptSourceFetcher func(ctx context.Context, job cidomain.ProjectJob, payload ScriptPayload, workDir string) error

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

func ExtractSourceArchive(content []byte, workDir string) (err error) {
	reader, err := zip.NewReader(bytes.NewReader(content), int64(len(content)))
	if err != nil {
		return fmt.Errorf("open source archive: %w", err)
	}
	root, err := filepath.Abs(workDir)
	if err != nil {
		return fmt.Errorf("resolve source archive target: %w", err)
	}
	rootHandle, err := os.OpenRoot(root)
	if err != nil {
		return fmt.Errorf("open source archive target: %w", err)
	}
	defer func() {
		if closeErr := rootHandle.Close(); closeErr != nil && err == nil {
			err = oops.In("runner_agent").With("root", root).Wrapf(closeErr, "close source archive target")
		}
	}()
	for _, file := range reader.File {
		relative, err := archiveTargetPath(file.Name)
		if err != nil {
			return err
		}
		info := file.FileInfo()
		if info.IsDir() {
			if mkdirErr := rootHandle.MkdirAll(relative, 0o750); mkdirErr != nil {
				return fmt.Errorf("create source directory: %w", mkdirErr)
			}
			continue
		}
		if !info.Mode().IsRegular() {
			continue
		}
		if parent := path.Dir(relative); parent != "." {
			if mkdirErr := rootHandle.MkdirAll(parent, 0o750); mkdirErr != nil {
				return fmt.Errorf("create source parent directory: %w", mkdirErr)
			}
		}
		rc, err := file.Open()
		if err != nil {
			return fmt.Errorf("open source archive entry: %w", err)
		}
		if err := writeArchiveFile(rootHandle, relative, rc, archiveFileMode(info.Mode().Perm())); err != nil {
			if closeErr := rc.Close(); closeErr != nil {
				return oops.In("runner_agent").With("target", relative).Wrapf(oops.Join(err, closeErr), "write source archive file and close archive entry")
			}
			return oops.In("runner_agent").With("target", relative).Wrapf(err, "write source archive file")
		}
		if err := rc.Close(); err != nil {
			return fmt.Errorf("close source archive entry: %w", err)
		}
	}
	return nil
}

func archiveTargetPath(name string) (string, error) {
	normalized := strings.Trim(strings.ReplaceAll(name, "\\", "/"), "/")
	if normalized == "" || normalized == "." || strings.HasPrefix(normalized, "../") || strings.Contains(normalized, "/../") || filepath.IsAbs(normalized) {
		return "", fmt.Errorf("invalid source archive path: %s", name)
	}
	return path.Clean(normalized), nil
}

func writeArchiveFile(root *os.Root, target string, reader io.Reader, mode os.FileMode) error {
	file, err := root.OpenFile(target, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, mode)
	if err != nil {
		return fmt.Errorf("create source archive file: %w", err)
	}
	if _, err := io.Copy(file, reader); err != nil {
		if closeErr := file.Close(); closeErr != nil {
			return oops.In("runner_agent").With("target", target).Wrapf(oops.Join(err, closeErr), "write source archive file and close target")
		}
		return fmt.Errorf("write source archive file: %w", err)
	}
	if err := file.Close(); err != nil {
		return fmt.Errorf("close source archive file: %w", err)
	}
	return nil
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

func runGit(ctx context.Context, dir string, args ...string) error {
	command := execabs.CommandContext(ctx, "git", args...)
	command.Dir = dir
	output, err := command.CombinedOutput()
	if err == nil {
		return nil
	}
	return fmt.Errorf("git %s: %w: %s", strings.Join(args, " "), err, strings.TrimSpace(string(output)))
}

func archiveFileMode(mode os.FileMode) os.FileMode {
	if mode&0o111 != 0 {
		return 0o700
	}
	return 0o600
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
			if err := b.stream(nil); err != nil {
				return len(p), oops.In("runner_agent").Wrapf(err, "stream truncated job output")
			}
		}
		return len(p), nil
	}
	if len(p) > remaining {
		b.truncated = true
		b.truncatedSent = true
		captured := p[:remaining]
		if _, err := b.buffer.Write(captured); err != nil {
			return len(p), oops.In("runner_agent").With("remaining", remaining).Wrapf(err, "capture truncated job output")
		}
		if err := b.stream(captured); err != nil {
			return len(p), oops.In("runner_agent").Wrapf(err, "stream captured truncated job output")
		}
		return len(p), nil
	}
	n, err := b.buffer.Write(p)
	if streamErr := b.stream(p); streamErr != nil {
		if err != nil {
			return n, oops.In("runner_agent").Wrapf(oops.Join(err, streamErr), "capture and stream job output")
		}
		return n, oops.In("runner_agent").Wrapf(streamErr, "stream job output")
	}
	if err != nil {
		return n, oops.In("runner_agent").Wrapf(err, "capture job output")
	}
	return n, nil
}

func (b *cappedBuffer) String() string {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.buffer.String()
}

func (b *cappedBuffer) stream(chunk []byte) error {
	if b.traceStreamer == nil {
		return nil
	}
	if len(chunk) == 0 && !b.truncated {
		return nil
	}
	duration := int64(0)
	if !b.started.IsZero() {
		duration = time.Since(b.started).Milliseconds()
	}
	if err := b.traceStreamer(b.ctx, string(chunk), b.truncated, duration); err != nil {
		return oops.In("runner_agent").With("truncated", b.truncated, "duration_millis", duration).Wrapf(err, "stream job trace")
	}
	return nil
}
