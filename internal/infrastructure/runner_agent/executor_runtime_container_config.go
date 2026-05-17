package runneragent

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"

	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/client"
	"github.com/samber/oops"

	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
)

func containerBinds(cfg Config, workDir string) ([]string, error) {
	containerDir := filepath.Clean(strings.TrimSpace(cfg.DockerWorkDir))
	if containerDir == "" {
		containerDir = "/workspace"
	}
	hostPath, err := filepath.Abs(workDir)
	if err != nil {
		return nil, oops.In("runner_agent").With("work_dir", workDir).Wrapf(err, "resolve container workdir")
	}
	return []string{filepath.ToSlash(hostPath) + ":" + containerDir}, nil
}

func containerResources(cfg Config) (container.Resources, error) {
	resources := container.Resources{}
	if err := applyContainerMemoryLimit(&resources, cfg.DockerMemoryLimit); err != nil {
		return resources, err
	}
	if err := applyContainerCPULimit(&resources, cfg.DockerCPUs); err != nil {
		return resources, err
	}
	return resources, nil
}

func applyContainerMemoryLimit(resources *container.Resources, value string) error {
	memoryLimit := strings.TrimSpace(value)
	if memoryLimit == "" {
		return nil
	}
	memory, err := parseByteSize(memoryLimit)
	if err != nil {
		return oops.In("runner_agent").With("memory_limit", memoryLimit).Wrapf(err, "parse docker memory limit")
	}
	resources.Memory = memory
	return nil
}

func applyContainerCPULimit(resources *container.Resources, value string) error {
	cpuLimit := strings.TrimSpace(value)
	if cpuLimit == "" {
		return nil
	}
	nanoCPU, err := parseCPUs(cpuLimit)
	if err != nil {
		return oops.In("runner_agent").With("cpus", cpuLimit).Wrapf(err, "parse docker cpu limit")
	}
	resources.NanoCPUs = nanoCPU
	return nil
}

func containerNetworkMode(cfg Config) string {
	if cfg.DockerHostNetwork {
		return "host"
	}
	return strings.TrimSpace(cfg.DockerNetwork)
}

func containerRuntimeClient(ctx context.Context, cfg Config, runtimeName, endpoint string) (*client.Client, error) {
	if runtimeName == "" {
		return nil, oops.In("runner_agent").New("runner container runtime is not configured")
	}
	apiHost := resolveContainerRuntimeHost(cfg, runtimeName, endpoint)
	if apiHost == "" {
		return nil, oops.In("runner_agent").With("runtime", runtimeName).New("runner container runtime endpoint is not configured")
	}
	apiClient, err := client.NewClientWithOpts(
		client.WithHost(apiHost),
		client.WithAPIVersionNegotiation(),
	)
	if err != nil {
		return nil, oops.In("runner_agent").With("runtime", runtimeName, "endpoint", apiHost).Wrapf(err, "create container runtime client")
	}
	if _, err := apiClient.ServerVersion(ctx); err != nil {
		closeRuntimeClient(apiClient)
		return nil, oops.In("runner_agent").With("runtime", runtimeName, "endpoint", apiHost).Wrapf(err, "probe container runtime")
	}
	return apiClient, nil
}

func resolveContainerRuntimeHost(cfg Config, runtimeName, endpoint string) string {
	apiHost := strings.TrimSpace(endpoint)
	if apiHost != "" {
		return apiHost
	}
	return resolveContainerRuntimeEndpoint(cfg, runtimeName)
}

func closeRuntimeClient(apiClient *client.Client) {
	if err := apiClient.Close(); err != nil {
		return
	}
}

func resolveContainerRuntimeEndpoint(cfg Config, runtimeName string) string {
	if strings.TrimSpace(cfg.ContainerRuntimeEndpoint) != "" {
		return strings.TrimSpace(cfg.ContainerRuntimeEndpoint)
	}
	switch runtimeName {
	case runnerExecutionModePodman:
		return defaultPodmanEndpoint()
	case runnerExecutionModeDocker:
		return defaultDockerEndpoint()
	case runnerExecutionModeContainerd:
		return defaultContainerdEndpoint()
	case runnerExecutionModeFirecracker:
		return strings.TrimSpace(cfg.FirecrackerSocket)
	default:
		return ""
	}
}

func defaultDockerEndpoint() string {
	if runtime.GOOS == "windows" {
		return "npipe:////./pipe/docker_engine"
	}
	return "unix:///var/run/docker.sock"
}

func defaultPodmanEndpoint() string {
	if runtime.GOOS != "linux" {
		return ""
	}
	if xdg := strings.TrimSpace(os.Getenv("XDG_RUNTIME_DIR")); xdg != "" {
		return "unix://" + filepath.ToSlash(filepath.Join(xdg, "podman", "podman.sock"))
	}
	return "unix:///run/user/" + strconv.Itoa(os.Getuid()) + "/podman/podman.sock"
}

func defaultContainerdEndpoint() string {
	if runtime.GOOS != "linux" {
		return ""
	}
	return "unix:///run/containerd/containerd.sock"
}

func parseByteSize(value string) (int64, error) {
	clean, multiplier := normalizeByteSize(value)
	parsed, err := strconv.ParseFloat(clean, 64)
	if err != nil {
		return 0, fmt.Errorf("invalid memory limit: %q", value)
	}
	if parsed < 0 {
		return 0, fmt.Errorf("memory limit must be positive: %q", value)
	}
	return int64(parsed * multiplier), nil
}

func normalizeByteSize(value string) (string, float64) {
	clean := strings.TrimSpace(strings.ToLower(value))
	if clean == "" {
		return "0", 1
	}
	unit := clean[len(clean)-1]
	multiplier := float64(1)
	switch unit {
	case 'k':
		multiplier = 1024
	case 'm':
		multiplier = 1024 * 1024
	case 'g':
		multiplier = 1024 * 1024 * 1024
	case 't':
		multiplier = 1024 * 1024 * 1024 * 1024
	case 'p':
		multiplier = 1024 * 1024 * 1024 * 1024 * 1024
	case 'e':
		multiplier = 1024 * 1024 * 1024 * 1024 * 1024 * 1024
	default:
		return clean, multiplier
	}
	return strings.TrimSpace(clean[:len(clean)-1]), multiplier
}

func parseCPUs(value string) (int64, error) {
	parsed, err := strconv.ParseFloat(strings.TrimSpace(value), 64)
	if err != nil {
		return 0, fmt.Errorf("invalid cpu limit: %q", value)
	}
	if parsed < 0 {
		return 0, fmt.Errorf("cpu limit must be non-negative: %q", value)
	}
	return int64(parsed * 1e9), nil
}

func containerShellCommand(shell, script string) []string {
	switch normalizeShellName(shell) {
	case "powershell", "powershell.exe":
		return []string{"powershell", "-NoProfile", "-NonInteractive", "-Command", script}
	case "pwsh", "pwsh.exe":
		return []string{"pwsh", "-NoProfile", "-NonInteractive", "-Command", script}
	case "cmd", "cmd.exe":
		return []string{"cmd.exe", "/Q", "/D", "/C", script}
	case "bash":
		return []string{"bash", "-lc", script}
	default:
		return []string{"sh", "-lc", script}
	}
}

func dockerScriptEnvironment(job cidomain.ProjectJob, payload ScriptPayload) []string {
	env := make([]string, 0, len(payload.Env)+8)
	for key, value := range payload.Env {
		key = strings.TrimSpace(key)
		if key != "" {
			env = append(env, fmt.Sprintf("%s=%s", key, value))
		}
	}
	return append(env,
		fmt.Sprintf("GITY_JOB_ID=%d", job.ID),
		fmt.Sprintf("GITY_PROJECT_ID=%d", job.ProjectID),
		fmt.Sprintf("GITY_JOB_ATTEMPT=%d", job.Attempts),
		"GITY_PROJECT_FULL_PATH="+payload.ProjectFullPath,
		"GITY_REF_NAME="+payload.RefName,
		"GITY_COMMIT_SHA="+payload.CommitSHA,
	)
}
