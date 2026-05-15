package runneragent_test

import (
	"context"
	"encoding/json"
	"runtime"
	"strings"
	"testing"
	"time"

	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	runneragent "github.com/lyonbrown4d/gity/internal/infrastructure/runner_agent"
)

func TestExecuteScriptJobFailure(t *testing.T) {
	t.Parallel()

	script := []string{"echo boom", "exit 7"}
	if runtime.GOOS == "windows" {
		script = []string{`Write-Output "boom"`, "exit 7"}
	}
	payload, err := json.Marshal(runneragent.ScriptPayload{
		Script:         script,
		TimeoutSeconds: 5,
	})
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}

	resultJSON, err := runneragent.ExecuteScriptJob(context.Background(), runneragent.Config{
		WorkDir:        t.TempDir(),
		LeaseSeconds:   30,
		MaxOutputBytes: 1024,
	}, cidomain.ProjectJob{
		ID:        43,
		ProjectID: 7,
		Kind:      "script",
		Payload:   string(payload),
		Attempts:  1,
	})
	if err == nil {
		t.Fatalf("expected script failure")
	}
	var result runneragent.ScriptResult
	if err := json.Unmarshal([]byte(resultJSON), &result); err != nil {
		t.Fatalf("unmarshal result: %v", err)
	}
	if result.ExitCode != 7 || !strings.Contains(result.Output, "boom") {
		t.Fatalf("unexpected failure result: %+v", result)
	}
}

func TestExecuteScriptJobCancellation(t *testing.T) {
	t.Parallel()

	script := []string{"sleep 5"}
	if runtime.GOOS == "windows" {
		script = []string{`Start-Sleep -Seconds 5`}
	}
	payload, err := json.Marshal(runneragent.ScriptPayload{
		Script:         script,
		TimeoutSeconds: 10,
	})
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}

	cancelAt := time.Now().Add(1200 * time.Millisecond)
	resultJSON, err := runneragent.ExecuteScriptJobWithChecker(context.Background(), runneragent.Config{
		WorkDir:        t.TempDir(),
		LeaseSeconds:   30,
		MaxOutputBytes: 1024,
	}, cidomain.ProjectJob{
		ID:        101,
		ProjectID: 7,
		Kind:      "script",
		Payload:   string(payload),
		Attempts:  1,
	}, func(_ context.Context) (bool, error) {
		if time.Now().After(cancelAt) {
			return true, nil
		}
		return false, nil
	})
	if err == nil {
		t.Fatalf("expected script cancellation")
	}
	var result runneragent.ScriptResult
	if err := json.Unmarshal([]byte(resultJSON), &result); err != nil {
		t.Fatalf("unmarshal result: %v", err)
	}
	if result.ExitCode != -1 {
		t.Fatalf("unexpected cancel result: %+v", result)
	}
}

func TestConfigFromEnvArgs(t *testing.T) {
	t.Setenv("GITY_RUNNER_TOKEN", "from-env")
	t.Setenv("GITY_RUNNER_URL", "http://gity.local/v1")
	t.Setenv("GITY_RUNNER_EXECUTION_MODE", "podman")
	t.Setenv("GITY_RUNNER_CONTAINER_RUNTIME", "firecracker")
	t.Setenv("GITY_RUNNER_CONTAINER_RUNTIME_ENDPOINT", "unix:///tmp/podman.sock")
	t.Setenv("GITY_RUNNER_CONTAINER_IMAGE", "alpine:3")
	t.Setenv("GITY_RUNNER_CONTAINER_NETWORK", "gity")
	t.Setenv("GITY_RUNNER_CONTAINER_HOST_NETWORK", "true")
	t.Setenv("GITY_RUNNER_CONTAINER_MEMORY", "1024m")
	t.Setenv("GITY_RUNNER_CONTAINER_CPUS", "2")
	t.Setenv("GITY_RUNNER_FIRECRACKER_SOCKET", "/tmp/fc.sock")

	cfg, err := runneragent.ConfigFromEnv([]string{"-token", "from-flag", "-poll-interval", "2s"})
	if err != nil {
		t.Fatalf("config from env args: %v", err)
	}
	if cfg.Token != "from-flag" || cfg.ServerURL != "http://gity.local/v1" || cfg.PollInterval != 2*time.Second {
		t.Fatalf("unexpected config: %+v", cfg)
	}
	if cfg.ExecutionMode != "podman" || cfg.ContainerRuntime != "firecracker" {
		t.Fatalf("unexpected runtime config: %+v", cfg)
	}
	if cfg.ContainerRuntimeEndpoint != "unix:///tmp/podman.sock" || cfg.DockerImage != "alpine:3" || cfg.DockerNetwork != "gity" {
		t.Fatalf("unexpected container config: %+v", cfg)
	}
	if !cfg.DockerHostNetwork || cfg.DockerMemoryLimit != "1024m" || cfg.DockerCPUs != "2" || cfg.FirecrackerSocket != "/tmp/fc.sock" {
		t.Fatalf("unexpected config: %+v", cfg)
	}
}

func TestExecuteScriptJobDockerModeMissingImage(t *testing.T) {
	t.Parallel()

	payload, err := json.Marshal(runneragent.ScriptPayload{
		ExecutionMode:  "docker",
		Script:         []string{"echo no-image"},
		TimeoutSeconds: 5,
	})
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}
	_, err = runneragent.ExecuteScriptJob(context.Background(), runneragent.Config{
		WorkDir:        t.TempDir(),
		LeaseSeconds:   30,
		MaxOutputBytes: 1024,
	ExecutionMode:  "docker",
	}, cidomain.ProjectJob{
		ID:        77,
		ProjectID: 7,
		Kind:      "script",
		Payload:   string(payload),
		Attempts:  1,
	})
	if err == nil {
		t.Fatal("expected docker mode to fail when no image is configured")
	}
}
