package runneragent

import (
	"context"
	"errors"
	"fmt"
	"io"
	"path/filepath"
	"os"
	"runtime"
	"strconv"
	"strings"
	"time"

	"github.com/docker/docker/api/types/container"
	dockerimage "github.com/docker/docker/api/types/image"
	"github.com/docker/docker/pkg/stdcopy"
	"github.com/docker/docker/client"
	"github.com/samber/oops"

	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
)

type scriptRunner interface {
	run(ctx context.Context, cfg Config, job cidomain.ProjectJob, payload ScriptPayload, workDir string, output *cappedBuffer) error
}

type hostScriptRunner struct{}

type dockerScriptRunner struct {
	endpoint string
}

type podmanScriptRunner struct {
	endpoint string
}

type containerdScriptRunner struct {
	endpoint string
}

type firecrackerScriptRunner struct {
	socket string
}

func resolveScriptRunner(cfg Config, payload ScriptPayload) (scriptRunner, error) {
	mode := resolveExecutionMode(cfg, payload)
	switch mode {
	case runnerExecutionModeHost:
		return hostScriptRunner{}, nil
	case runnerExecutionModeDocker, runnerExecutionModePodman:
		endpoint := resolveContainerRuntimeEndpoint(cfg, mode)
		if endpoint == "" {
			return nil, oops.In("runner_agent").With("runtime", mode).New("runner container runtime endpoint is required")
		}
		if image := resolveContainerImage(cfg, payload); image == "" {
			return nil, fmt.Errorf("runner execution mode is %s but no image is configured", mode)
		}
		if mode == runnerExecutionModeDocker {
			return dockerScriptRunner{endpoint: endpoint}, nil
		}
		return podmanScriptRunner{endpoint: endpoint}, nil
	case runnerExecutionModeContainerd:
		endpoint := resolveContainerRuntimeEndpoint(cfg, mode)
		if endpoint == "" {
			return nil, oops.In("runner_agent").With("runtime", mode).New("runner container runtime endpoint is required")
		}
		if image := resolveContainerImage(cfg, payload); image == "" {
			return nil, fmt.Errorf("runner execution mode is %s but no image is configured", mode)
		}
		return containerdScriptRunner{endpoint: endpoint}, nil
	case runnerExecutionModeFirecracker:
		socket := strings.TrimSpace(cfg.FirecrackerSocket)
		if socket == "" {
			return nil, oops.In("runner_agent").With("runtime", mode).New("firecracker socket is required")
		}
		if image := resolveContainerImage(cfg, payload); image == "" {
			return nil, fmt.Errorf("runner execution mode is %s but no image is configured", mode)
		}
		return firecrackerScriptRunner{socket: socket}, nil
	default:
		return nil, fmt.Errorf("unsupported runner execution mode: %s", mode)
	}
}

func (runner containerdScriptRunner) run(ctx context.Context, cfg Config, job cidomain.ProjectJob, payload ScriptPayload, workDir string, output *cappedBuffer) error {
	imageRef := resolveContainerImage(cfg, payload)
	command := strings.Join(payload.Script, "\n")
	shellCommand := containerShellCommand(payload.Shell, command)
	runtimeClient, err := containerRuntimeClient(cfg, runnerExecutionModeContainerd, runner.endpoint)
	if err != nil {
		return err
	}
	defer runtimeClient.Close()
	if imageRef == "" {
		return oops.In("runner_agent").With("runtime", runnerExecutionModeContainerd).New("no container image configured")
	}
	if err := ensureContainerImage(ctx, runtimeClient, imageRef); err != nil {
		return err
	}

	return runContainerScriptJob(ctx, cfg, job, payload, imageRef, shellCommand, workDir, runner.endpoint, runtimeClient, output, runnerExecutionModeContainerd)
}

func (runner firecrackerScriptRunner) run(_ context.Context, _ Config, _ cidomain.ProjectJob, _ ScriptPayload, _ string, _ *cappedBuffer) error {
	return runFirecrackerScriptJob(runner.socket)
}

func resolveExecutionMode(cfg Config, payload ScriptPayload) string {
	mode := normalizeExecutionMode(payload.ExecutionMode)
	if mode == "" {
		mode = normalizeExecutionMode(cfg.ExecutionMode)
	}
	if mode == "" {
		image := resolveContainerImage(cfg, payload)
		if image != "" {
			runtimeName := strings.TrimSpace(cfg.ContainerRuntime)
			if runtimeName == "" {
				runtimeName = runnerExecutionModeDocker
			}
			return runtimeName
		}
		return runnerExecutionModeHost
	}
	return mode
}

func resolveContainerImage(cfg Config, payload ScriptPayload) string {
	image := strings.TrimSpace(payload.Image)
	if image != "" {
		return image
	}
	return strings.TrimSpace(cfg.DockerImage)
}

func (hostScriptRunner) run(ctx context.Context, _ Config, job cidomain.ProjectJob, payload ScriptPayload, workDir string, output *cappedBuffer) error {
	command := scriptCommand(ctx, payload.Shell, strings.Join(payload.Script, "\n"))
	command.Dir = workDir
	command.Env = scriptEnvironment(job, payload)
	command.Stdout = output
	command.Stderr = output
	return command.Run()
}

func (runner dockerScriptRunner) run(ctx context.Context, cfg Config, job cidomain.ProjectJob, payload ScriptPayload, workDir string, output *cappedBuffer) error {
	imageRef := resolveContainerImage(cfg, payload)
	command := strings.Join(payload.Script, "\n")
	shellCommand := containerShellCommand(payload.Shell, command)
	runtimeClient, err := containerRuntimeClient(cfg, runnerExecutionModeDocker, runner.endpoint)
	if err != nil {
		return err
	}
	defer runtimeClient.Close()
	if imageRef == "" {
		return oops.In("runner_agent").With("runtime", runnerExecutionModeDocker).New("no container image configured")
	}
	if err := ensureContainerImage(ctx, runtimeClient, imageRef); err != nil {
		return err
	}

	return runContainerScriptJob(ctx, cfg, job, payload, imageRef, shellCommand, workDir, runner.endpoint, runtimeClient, output, runnerExecutionModeDocker)
}

func (runner podmanScriptRunner) run(ctx context.Context, cfg Config, job cidomain.ProjectJob, payload ScriptPayload, workDir string, output *cappedBuffer) error {
	imageRef := resolveContainerImage(cfg, payload)
	command := strings.Join(payload.Script, "\n")
	shellCommand := containerShellCommand(payload.Shell, command)
	runtimeClient, err := containerRuntimeClient(cfg, runnerExecutionModePodman, runner.endpoint)
	if err != nil {
		return err
	}
	defer runtimeClient.Close()
	if imageRef == "" {
		return oops.In("runner_agent").With("runtime", runnerExecutionModePodman).New("no container image configured")
	}
	if err := ensureContainerImage(ctx, runtimeClient, imageRef); err != nil {
		return err
	}

	return runContainerScriptJob(ctx, cfg, job, payload, imageRef, shellCommand, workDir, runner.endpoint, runtimeClient, output, runnerExecutionModePodman)
}

func ensureContainerImage(ctx context.Context, cli *client.Client, imageRef string) error {
	_, _, inspectErr := cli.ImageInspectWithRaw(ctx, imageRef)
	if inspectErr == nil {
		return nil
	}
	if client.IsErrNotFound(inspectErr) {
		pullResp, pullErr := cli.ImagePull(ctx, imageRef, dockerimage.PullOptions{})
		if pullErr != nil {
			return oops.In("runner_agent").With("image", imageRef).Wrapf(pullErr, "pull container image")
		}
		defer pullResp.Close()
		_, copyErr := io.Copy(io.Discard, pullResp)
		if copyErr != nil {
			return oops.In("runner_agent").With("image", imageRef).Wrapf(copyErr, "read container image pull output")
		}
		return nil
	}
	return oops.In("runner_agent").With("image", imageRef).Wrapf(inspectErr, "inspect container image")
}

func runContainerScriptJob(ctx context.Context, cfg Config, job cidomain.ProjectJob, payload ScriptPayload, image string, shellCommand []string, workDir string, endpoint string, cli *client.Client, output io.Writer, runtime string) error {
	binds, err := containerBinds(cfg, workDir)
	if err != nil {
		return err
	}
	resources, err := containerResources(cfg)
	if err != nil {
		return err
	}
	containerConfig := &container.Config{
		Image:      image,
		Cmd:        shellCommand,
		WorkingDir: cfg.DockerWorkDir,
		Env:        dockerScriptEnvironment(job, payload),
	}
	hostConfig := &container.HostConfig{
		AutoRemove:  true,
		Binds:       binds,
		NetworkMode: container.NetworkMode(containerNetworkMode(cfg)),
		Resources:   resources,
	}
	createResponse, err := cli.ContainerCreate(
		ctx,
		containerConfig,
		hostConfig,
		nil,
		nil,
		strings.TrimSpace(fmt.Sprintf("gity-script-%d-attempt-%d", job.ID, job.Attempts)),
	)
	if err != nil {
		return oops.In("runner_agent").With("image", image, "endpoint", endpoint, "runtime", runtime, "work_dir", workDir).Wrapf(err, "create container")
	}
	containerID := strings.TrimSpace(createResponse.ID)
	defer cleanupContainer(ctx, cli, containerID)
	if len(containerID) == 0 {
		return oops.In("runner_agent").With("image", image).New("container create returned empty container id")
	}

	attach, err := cli.ContainerAttach(ctx, containerID, container.AttachOptions{
		Stream: true,
		Stdout: true,
		Stderr: true,
	})
	if err != nil {
		return oops.In("runner_agent").With("container_id", containerID, "endpoint", endpoint, "runtime", runtime).Wrapf(err, "attach container")
	}
	defer attach.Close()
	if err := cli.ContainerStart(ctx, containerID, container.StartOptions{}); err != nil {
		return oops.In("runner_agent").With("container_id", containerID, "endpoint", endpoint, "runtime", runtime).Wrapf(err, "start container")
	}

	copyDone := make(chan error, 1)
	go func() {
		if _, copyErr := stdcopy.StdCopy(output, output, attach.Reader); copyErr != nil && !errors.Is(copyErr, io.EOF) {
			copyDone <- copyErr
			return
		}
		copyDone <- nil
	}()

	waitResponse, waitErr := waitContainer(ctx, cli, containerID, copyDone, endpoint)
	if waitErr != nil {
		return waitErr
	}
	if waitResponse.Error != nil {
		return oops.In("runner_agent").With("container_id", containerID, "status_code", waitResponse.StatusCode, "endpoint", endpoint, "error", waitResponse.Error.Message).New("container execution failed")
	}
	if waitResponse.StatusCode != 0 {
		return oops.In("runner_agent").With("container_id", containerID, "status_code", waitResponse.StatusCode, "endpoint", endpoint).New("container exited with non-zero status code")
	}
	return nil
}

func cleanupContainer(ctx context.Context, cli *client.Client, containerID string) {
	if containerID == "" {
		return
	}
	removeCtx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()
	_ = cli.ContainerRemove(removeCtx, containerID, container.RemoveOptions{Force: true})
}

func waitContainer(ctx context.Context, cli *client.Client, containerID string, copyDone <-chan error, endpoint string) (container.WaitResponse, error) {
	waitCh, errCh := cli.ContainerWait(ctx, containerID, container.WaitConditionNotRunning)
	select {
	case err := <-errCh:
		copyErr := <-copyDone
		if copyErr != nil && !errors.Is(copyErr, io.EOF) {
			return container.WaitResponse{}, oops.In("runner_agent").With("container_id", containerID, "endpoint", endpoint).Wrapf(copyErr, "read container output")
		}
		return container.WaitResponse{}, oops.In("runner_agent").With("container_id", containerID, "endpoint", endpoint).Wrapf(err, "wait container")
	case status := <-waitCh:
		copyErr := <-copyDone
		if copyErr != nil && !errors.Is(copyErr, io.EOF) {
			return container.WaitResponse{}, oops.In("runner_agent").With("container_id", containerID, "endpoint", endpoint).Wrapf(copyErr, "read container output")
		}
		return status, nil
	case <-ctx.Done():
		_ = cli.ContainerKill(context.Background(), containerID, "KILL")
		copyErr := <-copyDone
		if copyErr != nil && !errors.Is(copyErr, io.EOF) {
			return container.WaitResponse{}, oops.In("runner_agent").With("container_id", containerID, "endpoint", endpoint).Wrapf(copyErr, "read container output")
		}
		return container.WaitResponse{}, oops.In("runner_agent").With("container_id", containerID, "endpoint", endpoint).New("container execution canceled")
	}
}

func containerBinds(cfg Config, workDir string) ([]string, error) {
	containerDir := filepath.Clean(strings.TrimSpace(cfg.DockerWorkDir))
	if containerDir == "" {
		containerDir = "/workspace"
	}
	hostPath, err := filepath.Abs(workDir)
	if err != nil {
		return nil, oops.In("runner_agent").With("work_dir", workDir).Wrapf(err, "resolve container workdir")
	}
	hostPath = filepath.ToSlash(hostPath)
	return []string{hostPath + ":" + containerDir}, nil
}

func containerResources(cfg Config) (container.Resources, error) {
	resources := container.Resources{}
	memoryLimit := strings.TrimSpace(cfg.DockerMemoryLimit)
	if memoryLimit != "" {
		memory, err := parseByteSize(memoryLimit)
		if err != nil {
			return resources, oops.In("runner_agent").With("memory_limit", memoryLimit).Wrapf(err, "parse docker memory limit")
		}
		resources.Memory = memory
	}
	cpuLimit := strings.TrimSpace(cfg.DockerCPUs)
	if cpuLimit != "" {
		nanoCPU, err := parseCPUs(cpuLimit)
		if err != nil {
			return resources, oops.In("runner_agent").With("cpus", cpuLimit).Wrapf(err, "parse docker cpu limit")
		}
		resources.NanoCPUs = nanoCPU
	}
	return resources, nil
}

func containerNetworkMode(cfg Config) string {
	if cfg.DockerHostNetwork {
		return "host"
	}
	return strings.TrimSpace(cfg.DockerNetwork)
}

func containerRuntimeClient(cfg Config, runtimeName string, endpoint string) (*client.Client, error) {
	if runtimeName == "" {
		return nil, oops.In("runner_agent").New("runner container runtime is not configured")
	}
	apiHost := strings.TrimSpace(endpoint)
	if apiHost == "" {
		apiHost = resolveContainerRuntimeEndpoint(cfg, runtimeName)
	}
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
	_, versionErr := apiClient.ServerVersion(context.Background())
	if versionErr != nil {
		_ = apiClient.Close()
		return nil, oops.In("runner_agent").With("runtime", runtimeName, "endpoint", apiHost).Wrapf(versionErr, "probe container runtime")
	}
	return apiClient, nil
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
	uid := os.Getuid()
	return "unix:///run/user/" + strconv.Itoa(uid) + "/podman/podman.sock"
}

func defaultContainerdEndpoint() string {
	if runtime.GOOS != "linux" {
		return ""
	}
	return "unix:///run/containerd/containerd.sock"
}

func parseByteSize(value string) (int64, error) {
	clean := strings.TrimSpace(strings.ToLower(value))
	if clean == "" {
		return 0, nil
	}
	unit := clean[len(clean)-1]
	multiplier := float64(1)
	switch unit {
	case 'k':
		clean = strings.TrimSpace(clean[:len(clean)-1])
		multiplier = 1024
	case 'm':
		clean = strings.TrimSpace(clean[:len(clean)-1])
		multiplier = 1024 * 1024
	case 'g':
		clean = strings.TrimSpace(clean[:len(clean)-1])
		multiplier = 1024 * 1024 * 1024
	case 't':
		clean = strings.TrimSpace(clean[:len(clean)-1])
		multiplier = 1024 * 1024 * 1024 * 1024
	case 'p':
		clean = strings.TrimSpace(clean[:len(clean)-1])
		multiplier = 1024 * 1024 * 1024 * 1024 * 1024
	case 'e':
		clean = strings.TrimSpace(clean[:len(clean)-1])
		multiplier = 1024 * 1024 * 1024 * 1024 * 1024 * 1024
	}
	parsed, err := strconv.ParseFloat(clean, 64)
	if err != nil {
		return 0, fmt.Errorf("invalid memory limit: %q", value)
	}
	if parsed < 0 {
		return 0, fmt.Errorf("memory limit must be positive: %q", value)
	}
	return int64(parsed * multiplier), nil
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
	normalized := normalizeShellName(shell)
	switch normalized {
	case "powershell", "powershell.exe":
		return []string{"powershell", "-NoProfile", "-NonInteractive", "-Command", script}
	case "pwsh", "pwsh.exe":
		return []string{"pwsh", "-NoProfile", "-NonInteractive", "-Command", script}
	case "cmd", "cmd.exe":
		return []string{"cmd.exe", "/Q", "/D", "/C", script}
	case "bash":
		return []string{"bash", "-lc", script}
	case "sh":
		fallthrough
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
	env = append(env,
		fmt.Sprintf("GITY_JOB_ID=%d", job.ID),
		fmt.Sprintf("GITY_PROJECT_ID=%d", job.ProjectID),
		fmt.Sprintf("GITY_JOB_ATTEMPT=%d", job.Attempts),
		"GITY_PROJECT_FULL_PATH="+payload.ProjectFullPath,
		"GITY_REF_NAME="+payload.RefName,
		"GITY_COMMIT_SHA="+payload.CommitSHA,
	)
	return env
}
