package runneragent

import (
	"context"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"time"

	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	"github.com/samber/oops"
)

const (
	firecrackerNotImplementedMessage   = "firecracker runner runtime is not implemented yet"
	firecrackerCompatNotSupportedError = "firecracker runner mode is only partially supported through container-runtime compatibility mode"
	firecrackerReachabilityTimeout   = 5 * time.Second
)

func runFirecrackerScriptJob(
	ctx context.Context,
	cfg Config,
	job cidomain.ProjectJob,
	payload ScriptPayload,
	workDir string,
	output *cappedBuffer,
	socket string,
) error {
	if err := validateFirecrackerSocket(socket); err != nil {
		return oops.In("runner_agent").With("runtime", runnerExecutionModeFirecracker).Wrapf(err, "validate firecracker socket")
	}
	if err := validateFirecrackerWorkspace(workDir); err != nil {
		return oops.In("runner_agent").With("runtime", runnerExecutionModeFirecracker, "work_dir", workDir).Wrapf(err, "validate firecracker workspace")
	}
	if err := pingFirecracker(ctx, socket); err != nil {
		return oops.In("runner_agent").With("runtime", runnerExecutionModeFirecracker, "socket", socket).Wrapf(err, "validate firecracker runtime")
	}

	imageRef := resolveContainerImage(cfg, payload)
	if imageRef == "" {
		return oops.In("runner_agent").With("runtime", runnerExecutionModeFirecracker).New("no container image configured")
	}

	if isFirecrackerCompatibleWithContainerRuntime(socket) {
		if runtime.GOOS == "windows" {
			return oops.In("runner_agent").With(
				"runtime", runnerExecutionModeFirecracker,
				"socket", socket,
			).New(firecrackerCompatNotSupportedError)
		}
		command := strings.Join(payload.Script, "\n")
		shellCommand := containerShellCommand(payload.Shell, command)
		runtimeClient, err := containerRuntimeClient(cfg, runnerExecutionModeContainerd, socket)
		if err != nil {
			return oops.In("runner_agent").With(
				"runtime", runnerExecutionModeFirecracker,
				"socket", socket,
				"image", imageRef,
			).Wrapf(err, "initialize firecracker container compatibility runtime client")
		}
		defer runtimeClient.Close()
		if err := ensureContainerImage(ctx, runtimeClient, imageRef); err != nil {
			return err
		}

		return runContainerScriptJob(
			ctx,
			cfg,
			job,
			payload,
			imageRef,
			shellCommand,
			workDir,
			socket,
			runtimeClient,
			output,
			runnerExecutionModeFirecracker,
		)
	}

	return oops.In("runner_agent").With(
		"runtime", runnerExecutionModeFirecracker,
		"socket", socket,
		"job_id", job.ID,
		"project_id", job.ProjectID,
		"attempt", job.Attempts,
		"image", resolveContainerImage(cfg, payload),
	).New(firecrackerNotImplementedMessage)
}

func isFirecrackerCompatibleWithContainerRuntime(socket string) bool {
	trimmed := strings.TrimSpace(socket)
	if trimmed == "" {
		return false
	}

	if strings.Contains(trimmed, "://") {
		parsed, err := url.Parse(trimmed)
		if err != nil {
			return false
		}
		switch strings.ToLower(parsed.Scheme) {
		case "unix":
			return filepath.Ext(parsed.Path) == ".sock"
		case "http", "https":
			return strings.HasSuffix(parsed.Host, ".sock") || strings.Contains(strings.ToLower(parsed.Host), "containerd")
		case "npipe":
			return false
		}
		return strings.HasSuffix(parsed.Path, ".sock")
	}

	if runtime.GOOS != "linux" {
		return false
	}
	return filepath.IsAbs(trimmed) && filepath.Ext(trimmed) == ".sock"
}

func validateFirecrackerWorkspace(workDir string) error {
	trimmed := strings.TrimSpace(workDir)
	if trimmed == "" {
		return fmt.Errorf("firecracker workspace is required")
	}
	absPath, err := filepath.Abs(trimmed)
	if err != nil {
		return oops.In("runner_agent").With("work_dir", trimmed).Wrapf(err, "resolve firecracker workspace")
	}
	info, err := os.Stat(absPath)
	if err != nil {
		return oops.In("runner_agent").With("work_dir", absPath).Wrapf(err, "resolve firecracker workspace")
	}
	if !info.IsDir() {
		return oops.In("runner_agent").With("work_dir", absPath).New("firecracker workspace is not a directory")
	}
	return nil
}

func pingFirecracker(ctx context.Context, socket string) error {
	httpClient, baseEndpoint, err := firecrackerHTTPClient(socket)
	if err != nil {
		return oops.In("runner_agent").With("socket", socket).Wrapf(err, "initialize firecracker runtime client")
	}
	request, err := http.NewRequestWithContext(
		ctx,
		http.MethodGet,
		baseEndpoint,
		nil,
	)
	if err != nil {
		return oops.In("runner_agent").With("socket", socket).Wrapf(err, "build firecracker runtime request")
	}
	response, err := httpClient.Do(request)
	if err != nil {
		return oops.In("runner_agent").With("socket", socket, "endpoint", baseEndpoint).Wrapf(err, "ping firecracker runtime")
	}
	defer response.Body.Close()

	_, _ = io.Copy(io.Discard, response.Body)
	return nil
}

func firecrackerHTTPClient(value string) (*http.Client, string, error) {
	trimmed := strings.TrimSpace(value)
	if trimmed == "" {
		return nil, "", fmt.Errorf("firecracker socket is required")
	}
	if !strings.Contains(trimmed, "://") {
		if runtime.GOOS != "linux" {
			return nil, "", fmt.Errorf("firecracker socket %q requires unix:/// path or npipe:// on windows", trimmed)
		}
		return firecrackerUnixHTTPClient(trimmed)
	}

	parsed, err := url.Parse(trimmed)
	if err != nil {
		return nil, "", fmt.Errorf("invalid firecracker socket URL: %w", err)
	}
	if strings.EqualFold(parsed.Scheme, "unix") {
		if parsed.Path == "" {
			return nil, "", fmt.Errorf("firecracker unix socket URL must include socket path: %q", trimmed)
		}
		return firecrackerUnixHTTPClient(parsed.Path)
	}
	if parsed.Scheme == "" || parsed.Host == "" {
		return nil, "", fmt.Errorf("firecracker socket URL must include host: %q", trimmed)
	}
	base := &url.URL{
		Scheme: parsed.Scheme,
		Host:   parsed.Host,
	}
	return &http.Client{Timeout: firecrackerReachabilityTimeout}, base.String(), nil
}

func firecrackerUnixHTTPClient(path string) (*http.Client, string, error) {
	if runtime.GOOS != "linux" {
		return nil, "", fmt.Errorf("unix firecracker sockets are supported on linux only")
	}
	if strings.TrimSpace(path) == "" {
		return nil, "", fmt.Errorf("firecracker socket path is empty")
	}
	if !filepath.IsAbs(path) {
		return nil, "", fmt.Errorf("firecracker socket path must be absolute: %q", path)
	}
	info, err := os.Stat(path)
	if err != nil {
		return nil, "", oops.In("runner_agent").With("socket", path).Wrapf(err, "inspect firecracker socket")
	}
	if info.IsDir() {
		return nil, "", fmt.Errorf("firecracker socket is a directory: %q", path)
	}
	client := &http.Client{
		Transport: &http.Transport{
			DialContext: func(ctx context.Context, _, _ string) (net.Conn, error) {
				return (&net.Dialer{}).DialContext(ctx, "unix", path)
			},
			ForceAttemptHTTP2: false,
		},
		Timeout: firecrackerReachabilityTimeout,
	}
	return client, "http://localhost/", nil
}

func validateFirecrackerSocket(socket string) error {
	if strings.TrimSpace(socket) == "" {
		return fmt.Errorf("firecracker socket is required")
	}
	if runtime.GOOS == "windows" {
		if strings.HasPrefix(strings.ToLower(strings.TrimSpace(socket)), "npipe://") {
			return nil
		}
		parsed, err := url.Parse(socket)
		if err != nil {
			return fmt.Errorf("invalid firecracker socket URL: %w", err)
		}
		if parsed.Host == "" {
			return fmt.Errorf("windows firecracker socket requires npipe, http, or https endpoint: %q", strings.TrimSpace(socket))
		}
		scheme := strings.ToLower(parsed.Scheme)
		if scheme == "http" || scheme == "https" {
			return nil
		}
		return fmt.Errorf("windows firecracker socket requires npipe, http, or https endpoint: %q", strings.TrimSpace(socket))
	}
	if strings.Contains(socket, "://") {
		if _, err := url.Parse(socket); err != nil {
			return fmt.Errorf("invalid firecracker socket URL: %w", err)
		}
		return nil
	}
	if !strings.HasPrefix(socket, "/") {
		return fmt.Errorf("firecracker socket path must be absolute unix socket path or npipe URL")
	}
	return nil
}
