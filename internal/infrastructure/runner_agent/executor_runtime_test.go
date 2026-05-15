package runneragent

import (
	"context"
	"runtime"
	"strings"
	"testing"

	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
)

func TestResolveExecutionMode(t *testing.T) {
	t.Parallel()

	cfg := Config{
		ExecutionMode:    "host",
		ContainerRuntime: "docker",
	}
	hostOnlyPayload := ScriptPayload{}
	if got := resolveExecutionMode(cfg, hostOnlyPayload); got != runnerExecutionModeHost {
		t.Fatalf("expected host mode, got %q", got)
	}

	if got := resolveExecutionMode(Config{
		ExecutionMode:    "",
		ContainerRuntime: "docker",
		DockerImage:      "golang:1.22",
	}, hostOnlyPayload); got != runnerExecutionModeDocker {
		t.Fatalf("expected docker mode when default image is configured, got %q", got)
	}

	if got := resolveExecutionMode(Config{}, ScriptPayload{Image: "alpine"}); got != runnerExecutionModeDocker {
		t.Fatalf("expected docker mode when image is set and execution mode not explicitly configured, got %q", got)
	}

	if got := resolveExecutionMode(Config{
		ExecutionMode:    "docker",
		ContainerRuntime: "docker",
		DockerImage:      "alpine",
	}, hostOnlyPayload); got != runnerExecutionModeDocker {
		t.Fatalf("expected explicit docker mode from config, got %q", got)
	}

	if got := resolveExecutionMode(Config{
		ExecutionMode:    "podman",
		ContainerRuntime: "podman",
		DockerImage:      "alpine",
	}, hostOnlyPayload); got != runnerExecutionModePodman {
		t.Fatalf("expected podman mode from config, got %q", got)
	}

	if got := resolveExecutionMode(Config{
		ExecutionMode:    "docker",
		DockerImage:      "",
	}, ScriptPayload{Image: "alpine"}); got != runnerExecutionModeDocker {
		t.Fatalf("expected docker mode from explicit config even with payload image, got %q", got)
	}

	if got := resolveExecutionMode(Config{
		ExecutionMode: "containered",
	}, hostOnlyPayload); got != runnerExecutionModeContainerd {
		t.Fatalf("expected containered alias to be normalized to containerd, got %q", got)
	}

	if got := resolveExecutionMode(Config{
		ExecutionMode:    "",
		ContainerRuntime: runnerExecutionModeFirecracker,
		DockerImage:      "alpine:3",
	}, ScriptPayload{}); got != runnerExecutionModeFirecracker {
		t.Fatalf("expected firecracker when container runtime is firecracker and image is set, got %q", got)
	}

	if got := resolveExecutionMode(Config{
		ExecutionMode:    "host",
		ContainerRuntime: "podman",
		DockerImage:      "alpine:3",
	}, ScriptPayload{ExecutionMode: "containerd"}); got != runnerExecutionModeContainerd {
		t.Fatalf("expected payload execution mode to override config execution mode, got %q", got)
	}

	if got := resolveExecutionMode(Config{
		ExecutionMode:    "",
		ContainerRuntime: "podman",
		DockerImage:      "alpine:3",
	}, ScriptPayload{ExecutionMode: "host"}); got != runnerExecutionModeHost {
		t.Fatalf("expected payload host mode to override image fallback, got %q", got)
	}

	if got := resolveExecutionMode(Config{
		ExecutionMode:    "DOCKER",
		ContainerRuntime: "docker",
	}, ScriptPayload{}); got != runnerExecutionModeDocker {
		t.Fatalf("expected execution mode to be normalized, got %q", got)
	}
}

func TestResolveContainerRuntimeEndpoint(t *testing.T) {
	t.Parallel()

	if got := resolveContainerRuntimeEndpoint(Config{
		ContainerRuntimeEndpoint: "unix:///var/run/custom.sock",
	}, runnerExecutionModeDocker); got != "unix:///var/run/custom.sock" {
		t.Fatalf("expected global container runtime endpoint override, got %q", got)
	}

	if got := resolveContainerRuntimeEndpoint(Config{
		ContainerRuntime: "podman",
	}, runnerExecutionModePodman); got == "" {
		if runtime.GOOS == "linux" {
			t.Fatalf("expected resolved podman runtime endpoint")
		}
		if got != "" {
			t.Fatalf("expected empty podman runtime endpoint on non-linux")
		}
	}

	if got := resolveContainerRuntimeEndpoint(Config{
		ContainerRuntime: "containerd",
	}, runnerExecutionModeContainerd); got == "" {
		if runtime.GOOS == "linux" {
			t.Fatalf("expected resolved containerd runtime endpoint")
		}
	}
}

func TestResolveContainerImage(t *testing.T) {
	t.Parallel()

	cfg := Config{
		DockerImage: "alpine",
	}
	if got := resolveContainerImage(cfg, ScriptPayload{}); got != "alpine" {
		t.Fatalf("expected default image, got %q", got)
	}
	if got := resolveContainerImage(cfg, ScriptPayload{Image: "golang:1.22"}); got != "golang:1.22" {
		t.Fatalf("expected payload image, got %q", got)
	}
}

func TestContainerResourceLimits(t *testing.T) {
	t.Parallel()

	memory, err := parseByteSize("512m")
	if err != nil {
		t.Fatalf("parse memory: %v", err)
	}
	if memory != 536870912 {
		t.Fatalf("unexpected memory value, got %d", memory)
	}

	nanoCPU, err := parseCPUs("1.5")
	if err != nil {
		t.Fatalf("parse cpus: %v", err)
	}
	if nanoCPU != 1500000000 {
		t.Fatalf("unexpected cpu value, got %d", nanoCPU)
	}
}

func TestResolveScriptRunnerWithoutImageReturnsError(t *testing.T) {
	t.Parallel()

	_, err := resolveScriptRunner(Config{
		ExecutionMode:    "docker",
		ContainerRuntime: "docker",
	}, ScriptPayload{})
	if err == nil {
		t.Fatal("expected error when no docker image is configured")
	}
}

func TestResolveScriptRunnerReturnsDockerAndPodmanRunners(t *testing.T) {
	t.Parallel()

	dockerRunner, err := resolveScriptRunner(Config{
		ExecutionMode:    "docker",
		ContainerRuntime: "docker",
		DockerImage:      "alpine:3",
	}, ScriptPayload{})
	if err != nil {
		t.Fatalf("resolve docker runner: %v", err)
	}
	if _, ok := dockerRunner.(dockerScriptRunner); !ok {
		t.Fatalf("expected docker script runner, got %T", dockerRunner)
	}

	podmanRunner, err := resolveScriptRunner(Config{
		ExecutionMode:    "podman",
		ContainerRuntime: "podman",
		ContainerRuntimeEndpoint: "unix:///tmp/podman.sock",
		DockerImage:      "alpine:3",
	}, ScriptPayload{})
	if err != nil {
		t.Fatalf("resolve podman runner: %v", err)
	}
	if _, ok := podmanRunner.(podmanScriptRunner); !ok {
		t.Fatalf("expected podman script runner, got %T", podmanRunner)
	}
}

func TestScriptRunnerForPodmanWithoutImageReturnsError(t *testing.T) {
	t.Parallel()

	_, err := resolveScriptRunner(Config{
		ExecutionMode:    "podman",
		ContainerRuntime: "podman",
		ContainerRuntimeEndpoint: "unix:///tmp/podman.sock",
	}, ScriptPayload{})
	if err == nil || !strings.Contains(strings.ToLower(err.Error()), "no image") {
		t.Fatalf("expected podman image missing error, got %v", err)
	}
}

func TestScriptRunnerForDockerOrPodmanWithoutEndpointReturnsError(t *testing.T) {
	t.Parallel()

	_, err := resolveScriptRunner(Config{
		ExecutionMode:    "docker",
		ContainerRuntime: "docker",
		ContainerRuntimeEndpoint: "",
		DockerImage:      "alpine:3",
	}, ScriptPayload{})
	if err != nil {
		t.Fatalf("unexpected docker endpoint resolution error: %v", err)
	}

	_, err = resolveScriptRunner(Config{
		ExecutionMode:    "podman",
		ContainerRuntime: "podman",
		ContainerRuntimeEndpoint: "",
		DockerImage:      "alpine:3",
	}, ScriptPayload{})
	if err == nil {
		if runtime.GOOS != "linux" {
			t.Fatalf("expected podman endpoint required error on non-linux environment")
		}
		return
	}
	if runtime.GOOS == "linux" {
		t.Fatalf("unexpected podman endpoint resolution error on linux environment: %v", err)
	}
}

func TestScriptRunnerForContainerdAndFirecrackerRunFlow(t *testing.T) {
	t.Parallel()

	runner, err := resolveScriptRunner(Config{
		ExecutionMode:   "containerd",
		ContainerRuntime: runnerExecutionModeContainerd,
		DockerImage:      "alpine:3",
		ContainerRuntimeEndpoint: "unix:///run/containerd/containerd.sock",
	}, ScriptPayload{})
	if err != nil {
		t.Fatalf("expected containerd script runner to be returned, got error: %v", err)
	}
	if _, ok := runner.(containerdScriptRunner); !ok {
		t.Fatalf("expected containerd script runner, got %T", runner)
	}

	runner, err = resolveScriptRunner(Config{
		ExecutionMode:   "firecracker",
		ContainerRuntime: runnerExecutionModeContainerd,
		DockerImage:      "alpine:3",
		FirecrackerSocket: "/tmp/fc.sock",
	}, ScriptPayload{})
	if err != nil {
		t.Fatalf("expected firecracker script runner to be returned, got error: %v", err)
	}
	if _, ok := runner.(firecrackerScriptRunner); !ok {
		t.Fatalf("expected firecracker script runner, got %T", runner)
	}
	err = runner.run(context.Background(), Config{}, cidomain.ProjectJob{}, ScriptPayload{}, "", &cappedBuffer{})
	if err == nil || !strings.Contains(err.Error(), "firecracker runner runtime is not implemented yet") {
		t.Fatalf("expected firecracker not implemented error, got %v", err)
	}
}

func TestResolveScriptRunnerMissingContainerdEndpointOrImage(t *testing.T) {
	t.Parallel()

	_, err := resolveScriptRunner(Config{
		ExecutionMode:    "containerd",
		ContainerRuntime: runnerExecutionModeContainerd,
		DockerImage:      "alpine:3",
	}, ScriptPayload{})
	if err != nil && !strings.Contains(err.Error(), "container runtime endpoint is required") {
		t.Fatalf("expected containerd endpoint required error, got %v", err)
	}

	_, err = resolveScriptRunner(Config{
		ExecutionMode:    "containerd",
		ContainerRuntime: runnerExecutionModeContainerd,
		ContainerRuntimeEndpoint: "unix:///run/containerd/containerd.sock",
	}, ScriptPayload{})
	if err == nil || !strings.Contains(strings.ToLower(err.Error()), "no image") {
		t.Fatalf("expected containerd image missing error, got %v", err)
	}
}

func TestResolveScriptRunnerMissingFirecrackerSocketOrImage(t *testing.T) {
	t.Parallel()

	_, err := resolveScriptRunner(Config{
		ExecutionMode:    "firecracker",
		ContainerRuntime: runnerExecutionModeFirecracker,
		DockerImage:      "alpine:3",
	}, ScriptPayload{})
	if err == nil || !strings.Contains(err.Error(), "firecracker socket is required") {
		t.Fatalf("expected firecracker socket required error, got %v", err)
	}

	_, err = resolveScriptRunner(Config{
		ExecutionMode:    "firecracker",
		ContainerRuntime: runnerExecutionModeFirecracker,
		FirecrackerSocket: "/tmp/fc.sock",
	}, ScriptPayload{})
	if err == nil || !strings.Contains(strings.ToLower(err.Error()), "no image") {
		t.Fatalf("expected firecracker image missing error, got %v", err)
	}
}
