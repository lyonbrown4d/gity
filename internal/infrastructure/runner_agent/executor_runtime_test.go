package runneragent_test

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"runtime"
	"strings"
	"testing"

	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	runneragent "github.com/lyonbrown4d/gity/internal/infrastructure/runner_agent"
)

func TestResolveExecutionMode(t *testing.T) {
	t.Parallel()

	cases := []struct {
		name    string
		cfg     runneragent.Config
		payload runneragent.ScriptPayload
		want    string
	}{
		{name: "host config", cfg: runneragent.Config{ExecutionMode: "host", ContainerRuntime: "docker"}, want: "host"},
		{name: "default image", cfg: runneragent.Config{DockerImage: "golang:1.22", ContainerRuntime: "docker"}, want: "docker"},
		{name: "payload image", payload: runneragent.ScriptPayload{Image: "alpine"}, want: "docker"},
		{name: "explicit docker", cfg: runneragent.Config{ExecutionMode: "docker", DockerImage: "alpine"}, want: "docker"},
		{name: "explicit podman", cfg: runneragent.Config{ExecutionMode: "podman", DockerImage: "alpine"}, want: "podman"},
		{name: "firecracker runtime fallback", cfg: runneragent.Config{ContainerRuntime: "firecracker", DockerImage: "alpine:3"}, want: "firecracker"},
		{name: "payload override", cfg: runneragent.Config{ExecutionMode: "host", DockerImage: "alpine:3"}, payload: runneragent.ScriptPayload{ExecutionMode: "containerd"}, want: "containerd"},
		{name: "payload host override", cfg: runneragent.Config{ContainerRuntime: "podman", DockerImage: "alpine:3"}, payload: runneragent.ScriptPayload{ExecutionMode: "host"}, want: "host"},
		{name: "normalizes case", cfg: runneragent.Config{ExecutionMode: "DOCKER"}, want: "docker"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			assertEqual(t, runneragent.ResolveExecutionMode(tc.cfg, tc.payload), tc.want)
		})
	}
}

func TestResolveContainerRuntimeEndpoint(t *testing.T) {
	t.Parallel()

	got := runneragent.ResolveContainerRuntimeEndpoint(
		runneragent.Config{ContainerRuntimeEndpoint: "unix:///var/run/custom.sock"},
		"docker",
	)
	assertEqual(t, got, "unix:///var/run/custom.sock")

	podmanEndpoint := runneragent.ResolveContainerRuntimeEndpoint(runneragent.Config{ContainerRuntime: "podman"}, "podman")
	if runtime.GOOS == "linux" && podmanEndpoint == "" {
		t.Fatalf("expected resolved podman runtime endpoint")
	}

	containerdEndpoint := runneragent.ResolveContainerRuntimeEndpoint(runneragent.Config{ContainerRuntime: "containerd"}, "containerd")
	if runtime.GOOS == "linux" && containerdEndpoint == "" {
		t.Fatalf("expected resolved containerd runtime endpoint")
	}
}

func TestResolveContainerImage(t *testing.T) {
	t.Parallel()

	cfg := runneragent.Config{DockerImage: "alpine"}
	assertEqual(t, runneragent.ResolveContainerImage(cfg, runneragent.ScriptPayload{}), "alpine")
	assertEqual(t, runneragent.ResolveContainerImage(cfg, runneragent.ScriptPayload{Image: "golang:1.22"}), "golang:1.22")
}

func TestContainerResourceLimits(t *testing.T) {
	t.Parallel()

	memory, err := runneragent.ParseContainerByteSize("512m")
	if err != nil {
		t.Fatalf("parse memory: %v", err)
	}
	if memory != 536870912 {
		t.Fatalf("unexpected memory value, got %d", memory)
	}

	nanoCPU, err := runneragent.ParseContainerCPUs("1.5")
	if err != nil {
		t.Fatalf("parse cpus: %v", err)
	}
	if nanoCPU != 1500000000 {
		t.Fatalf("unexpected cpu value, got %d", nanoCPU)
	}
}

func TestResolveScriptRunnerWithoutImageReturnsError(t *testing.T) {
	t.Parallel()

	_, err := runneragent.ResolveScriptRunnerKind(runneragent.Config{ExecutionMode: "docker"}, runneragent.ScriptPayload{})
	if err == nil {
		t.Fatal("expected error when no docker image is configured")
	}
}

func TestResolveScriptRunnerReturnsContainerRunners(t *testing.T) {
	t.Parallel()

	cases := []struct {
		name string
		cfg  runneragent.Config
		want string
	}{
		{name: "docker", cfg: runneragent.Config{ExecutionMode: "docker", DockerImage: "alpine:3"}, want: "docker"},
		{name: "podman", cfg: runneragent.Config{ExecutionMode: "podman", DockerImage: "alpine:3", ContainerRuntimeEndpoint: "unix:///tmp/podman.sock"}, want: "podman"},
		{name: "containerd", cfg: runneragent.Config{ExecutionMode: "containerd", DockerImage: "alpine:3", ContainerRuntimeEndpoint: "unix:///run/containerd/containerd.sock"}, want: "containerd"},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			got, err := runneragent.ResolveScriptRunnerKind(tc.cfg, runneragent.ScriptPayload{})
			if err != nil {
				t.Fatalf("resolve script runner: %v", err)
			}
			assertEqual(t, got, tc.want)
		})
	}
}

func TestFirecrackerRuntimeCompatibilityDetection(t *testing.T) {
	t.Parallel()

	cases := []struct {
		socket string
		want   bool
	}{
		{socket: "unix:///run/containerd/containerd.sock", want: true},
		{socket: "http://containerd.localhost:8080/", want: true},
		{socket: "unix:///tmp/fc.sock/control", want: false},
		{socket: "npipe:///var/run/fc", want: false},
		{socket: "   ", want: false},
	}
	for _, tc := range cases {
		got := runneragent.IsFirecrackerCompatibleWithContainerRuntime(tc.socket)
		if got != tc.want {
			t.Fatalf("unexpected compatibility for %q: got %v want %v", tc.socket, got, tc.want)
		}
	}
}

func TestScriptRunnerForPodmanWithoutImageReturnsError(t *testing.T) {
	t.Parallel()

	_, err := runneragent.ResolveScriptRunnerKind(runneragent.Config{
		ExecutionMode:            "podman",
		ContainerRuntime:         "podman",
		ContainerRuntimeEndpoint: "unix:///tmp/podman.sock",
	}, runneragent.ScriptPayload{})
	if err == nil || !strings.Contains(strings.ToLower(err.Error()), "no image") {
		t.Fatalf("expected podman image missing error, got %v", err)
	}
}

func TestScriptRunnerForDockerOrPodmanWithoutEndpointReturnsError(t *testing.T) {
	t.Parallel()

	_, err := runneragent.ResolveScriptRunnerKind(runneragent.Config{ExecutionMode: "docker", DockerImage: "alpine:3"}, runneragent.ScriptPayload{})
	if err != nil {
		t.Fatalf("unexpected docker endpoint resolution error: %v", err)
	}

	_, err = runneragent.ResolveScriptRunnerKind(runneragent.Config{ExecutionMode: "podman", DockerImage: "alpine:3"}, runneragent.ScriptPayload{})
	if runtime.GOOS == "linux" && err != nil {
		t.Fatalf("unexpected podman endpoint resolution error on linux: %v", err)
	}
	if runtime.GOOS != "linux" && err == nil {
		t.Fatalf("expected podman endpoint required error on non-linux environment")
	}
}

func TestScriptRunnerForContainerdAndFirecrackerRunFlow(t *testing.T) {
	t.Parallel()

	kind, err := runneragent.ResolveScriptRunnerKind(runneragent.Config{
		ExecutionMode:            "containerd",
		ContainerRuntime:         "containerd",
		DockerImage:              "alpine:3",
		ContainerRuntimeEndpoint: "unix:///run/containerd/containerd.sock",
	}, runneragent.ScriptPayload{})
	if err != nil {
		t.Fatalf("expected containerd script runner, got error: %v", err)
	}
	assertEqual(t, kind, "containerd")

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusNoContent)
	}))
	defer server.Close()
	err = runFirecrackerNotImplemented(t, server.URL)
	if err == nil || !strings.Contains(err.Error(), "firecracker runner runtime is not implemented yet") {
		t.Fatalf("expected firecracker not implemented error, got %v", err)
	}
}

func TestResolveScriptRunnerMissingContainerdEndpointOrImage(t *testing.T) {
	t.Parallel()

	_, err := runneragent.ResolveScriptRunnerKind(runneragent.Config{ExecutionMode: "containerd", DockerImage: "alpine:3"}, runneragent.ScriptPayload{})
	if err != nil && !strings.Contains(err.Error(), "container runtime endpoint is required") {
		t.Fatalf("expected containerd endpoint required error, got %v", err)
	}

	_, err = runneragent.ResolveScriptRunnerKind(runneragent.Config{
		ExecutionMode:            "containerd",
		ContainerRuntimeEndpoint: "unix:///run/containerd/containerd.sock",
	}, runneragent.ScriptPayload{})
	if err == nil || !strings.Contains(strings.ToLower(err.Error()), "no image") {
		t.Fatalf("expected containerd image missing error, got %v", err)
	}
}

func TestResolveScriptRunnerMissingFirecrackerSocketOrImage(t *testing.T) {
	t.Parallel()

	_, err := runneragent.ResolveScriptRunnerKind(runneragent.Config{ExecutionMode: "firecracker", DockerImage: "alpine:3"}, runneragent.ScriptPayload{})
	if err == nil || !strings.Contains(err.Error(), "firecracker socket is required") {
		t.Fatalf("expected firecracker socket required error, got %v", err)
	}

	_, err = runneragent.ResolveScriptRunnerKind(runneragent.Config{ExecutionMode: "firecracker", FirecrackerSocket: "/tmp/fc.sock"}, runneragent.ScriptPayload{})
	if err == nil || !strings.Contains(strings.ToLower(err.Error()), "no image") {
		t.Fatalf("expected firecracker image missing error, got %v", err)
	}
}

func runFirecrackerNotImplemented(t *testing.T, socket string) error {
	t.Helper()

	payload, err := json.Marshal(runneragent.ScriptPayload{
		ExecutionMode: "firecracker",
		Image:         "alpine:3",
		Script:        []string{"echo ok"},
	})
	if err != nil {
		t.Fatalf("marshal script payload: %v", err)
	}
	_, err = runneragent.ExecuteScriptJob(context.Background(), runneragent.Config{
		WorkDir:           t.TempDir(),
		LeaseSeconds:      30,
		MaxOutputBytes:    1024,
		FirecrackerSocket: socket,
	}, cidomain.ProjectJob{
		ID:        42,
		ProjectID: 7,
		Kind:      "script",
		Payload:   string(payload),
		Attempts:  1,
	})
	if err != nil {
		return fmt.Errorf("execute firecracker script job: %w", err)
	}
	return nil
}

func assertEqual(t *testing.T, got, want string) {
	t.Helper()

	if got != want {
		t.Fatalf("got %q want %q", got, want)
	}
}
