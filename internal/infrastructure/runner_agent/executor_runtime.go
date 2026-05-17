package runneragent

import (
	"context"
	"fmt"
	"strings"

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
	case runnerExecutionModeDocker, runnerExecutionModePodman, runnerExecutionModeContainerd:
		return resolveContainerScriptRunner(cfg, payload, mode)
	case runnerExecutionModeFirecracker:
		return resolveFirecrackerScriptRunner(cfg, payload)
	default:
		return nil, fmt.Errorf("unsupported runner execution mode: %s", mode)
	}
}

func resolveContainerScriptRunner(cfg Config, payload ScriptPayload, mode string) (scriptRunner, error) {
	endpoint := resolveContainerRuntimeEndpoint(cfg, mode)
	if endpoint == "" {
		return nil, oops.In("runner_agent").With("runtime", mode).New("runner container runtime endpoint is required")
	}
	if image := resolveContainerImage(cfg, payload); image == "" {
		return nil, fmt.Errorf("runner execution mode is %s but no image is configured", mode)
	}
	switch mode {
	case runnerExecutionModeDocker:
		return dockerScriptRunner{endpoint: endpoint}, nil
	case runnerExecutionModePodman:
		return podmanScriptRunner{endpoint: endpoint}, nil
	default:
		return containerdScriptRunner{endpoint: endpoint}, nil
	}
}

func resolveFirecrackerScriptRunner(cfg Config, payload ScriptPayload) (scriptRunner, error) {
	socket := strings.TrimSpace(cfg.FirecrackerSocket)
	if socket == "" {
		return nil, oops.In("runner_agent").With("runtime", runnerExecutionModeFirecracker).New("firecracker socket is required")
	}
	if image := resolveContainerImage(cfg, payload); image == "" {
		return nil, fmt.Errorf("runner execution mode is %s but no image is configured", runnerExecutionModeFirecracker)
	}
	return firecrackerScriptRunner{socket: socket}, nil
}

func (runner containerdScriptRunner) run(ctx context.Context, cfg Config, job cidomain.ProjectJob, payload ScriptPayload, workDir string, output *cappedBuffer) error {
	return runDockerCompatibleScript(ctx, cfg, job, payload, workDir, output, runnerExecutionModeContainerd, runner.endpoint)
}

func (runner firecrackerScriptRunner) run(ctx context.Context, cfg Config, job cidomain.ProjectJob, payload ScriptPayload, workDir string, output *cappedBuffer) error {
	return runFirecrackerScriptJob(ctx, cfg, job, payload, workDir, output, runner.socket)
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
	if err := command.Run(); err != nil {
		return oops.In("runner_agent").With("work_dir", workDir).Wrapf(err, "run host script")
	}
	return nil
}

func (runner dockerScriptRunner) run(ctx context.Context, cfg Config, job cidomain.ProjectJob, payload ScriptPayload, workDir string, output *cappedBuffer) error {
	return runDockerCompatibleScript(ctx, cfg, job, payload, workDir, output, runnerExecutionModeDocker, runner.endpoint)
}

func (runner podmanScriptRunner) run(ctx context.Context, cfg Config, job cidomain.ProjectJob, payload ScriptPayload, workDir string, output *cappedBuffer) error {
	return runDockerCompatibleScript(ctx, cfg, job, payload, workDir, output, runnerExecutionModePodman, runner.endpoint)
}

func runDockerCompatibleScript(ctx context.Context, cfg Config, job cidomain.ProjectJob, payload ScriptPayload, workDir string, output *cappedBuffer, runtimeName, endpoint string) error {
	imageRef := resolveContainerImage(cfg, payload)
	command := strings.Join(payload.Script, "\n")
	shellCommand := containerShellCommand(payload.Shell, command)
	runtimeClient, err := containerRuntimeClient(ctx, cfg, runtimeName, endpoint)
	if err != nil {
		return err
	}
	defer closeRuntimeClient(runtimeClient)
	if imageRef == "" {
		return oops.In("runner_agent").With("runtime", runtimeName).New("no container image configured")
	}
	if err := ensureContainerImage(ctx, runtimeClient, imageRef); err != nil {
		return err
	}
	return runContainerScriptJob(ctx, cfg, job, payload, imageRef, shellCommand, workDir, endpoint, runtimeClient, output, runtimeName)
}
