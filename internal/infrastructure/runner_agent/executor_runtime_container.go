package runneragent

import (
	"context"
	"errors"
	"fmt"
	"io"
	"strings"
	"time"

	containerderrdefs "github.com/containerd/errdefs"
	dockertypes "github.com/docker/docker/api/types"
	"github.com/docker/docker/api/types/container"
	dockerimage "github.com/docker/docker/api/types/image"
	"github.com/docker/docker/client"
	"github.com/docker/docker/pkg/stdcopy"
	"github.com/samber/oops"

	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
)

type containerScriptJob struct {
	ctx          context.Context
	cfg          Config
	job          cidomain.ProjectJob
	payload      ScriptPayload
	image        string
	shellCommand []string
	workDir      string
	endpoint     string
	runtimeName  string
	cli          *client.Client
	output       io.Writer
}

func ensureContainerImage(ctx context.Context, cli *client.Client, imageRef string) error {
	if _, inspectErr := cli.ImageInspect(ctx, imageRef); inspectErr == nil {
		return nil
	} else if !containerderrdefs.IsNotFound(inspectErr) {
		return oops.In("runner_agent").With("image", imageRef).Wrapf(inspectErr, "inspect container image")
	}

	pullResp, err := cli.ImagePull(ctx, imageRef, dockerimage.PullOptions{})
	if err != nil {
		return oops.In("runner_agent").With("image", imageRef).Wrapf(err, "pull container image")
	}
	_, copyErr := io.Copy(io.Discard, pullResp)
	closeErr := pullResp.Close()
	if copyErr != nil {
		return oops.In("runner_agent").With("image", imageRef).Wrapf(copyErr, "read container image pull output")
	}
	if closeErr != nil {
		return oops.In("runner_agent").With("image", imageRef).Wrapf(closeErr, "close container image pull output")
	}
	return nil
}

func runContainerScriptJob(
	ctx context.Context,
	cfg Config,
	job cidomain.ProjectJob,
	payload ScriptPayload,
	image string,
	shellCommand []string,
	workDir, endpoint string,
	cli *client.Client,
	output io.Writer,
	runtimeName string,
) error {
	return containerScriptJob{
		ctx:          ctx,
		cfg:          cfg,
		job:          job,
		payload:      payload,
		image:        image,
		shellCommand: shellCommand,
		workDir:      workDir,
		endpoint:     endpoint,
		runtimeName:  runtimeName,
		cli:          cli,
		output:       output,
	}.run()
}

func (run containerScriptJob) run() error {
	containerID, err := run.createContainer()
	if err != nil {
		return err
	}
	defer cleanupContainer(run.ctx, run.cli, containerID)
	if containerID == "" {
		return oops.In("runner_agent").With("image", run.image).New("container create returned empty container id")
	}

	copyDone, err := run.attachStartAndCopy(containerID)
	if err != nil {
		return err
	}
	waitResponse, err := waitContainer(run.ctx, run.cli, containerID, copyDone, run.endpoint)
	if err != nil {
		return err
	}
	return run.checkContainerStatus(containerID, waitResponse)
}

func (run containerScriptJob) createContainer() (string, error) {
	containerConfig, hostConfig, err := run.containerConfigs()
	if err != nil {
		return "", err
	}
	createResponse, err := run.cli.ContainerCreate(
		run.ctx,
		containerConfig,
		hostConfig,
		nil,
		nil,
		containerName(run.job),
	)
	if err != nil {
		return "", oops.In("runner_agent").With(
			"image", run.image,
			"endpoint", run.endpoint,
			"runtime", run.runtimeName,
			"work_dir", run.workDir,
		).Wrapf(err, "create container")
	}
	return strings.TrimSpace(createResponse.ID), nil
}

func containerName(job cidomain.ProjectJob) string {
	return strings.TrimSpace(fmt.Sprintf("gity-script-%d-attempt-%d", job.ID, job.Attempts))
}

func (run containerScriptJob) containerConfigs() (*container.Config, *container.HostConfig, error) {
	binds, err := containerBinds(run.cfg, run.workDir)
	if err != nil {
		return nil, nil, err
	}
	resources, err := containerResources(run.cfg)
	if err != nil {
		return nil, nil, err
	}
	return &container.Config{
			Image:      run.image,
			Cmd:        run.shellCommand,
			WorkingDir: run.cfg.DockerWorkDir,
			Env:        dockerScriptEnvironment(run.job, run.payload),
		}, &container.HostConfig{
			AutoRemove:  true,
			Binds:       binds,
			NetworkMode: container.NetworkMode(containerNetworkMode(run.cfg)),
			Resources:   resources,
		}, nil
}

func (run containerScriptJob) attachStartAndCopy(containerID string) (<-chan error, error) {
	attach, err := run.cli.ContainerAttach(run.ctx, containerID, container.AttachOptions{
		Stream: true,
		Stdout: true,
		Stderr: true,
	})
	if err != nil {
		return nil, oops.In("runner_agent").With(
			"container_id", containerID,
			"endpoint", run.endpoint,
			"runtime", run.runtimeName,
		).Wrapf(err, "attach container")
	}
	if err := run.cli.ContainerStart(run.ctx, containerID, container.StartOptions{}); err != nil {
		closeHijackedResponse(attach)
		return nil, oops.In("runner_agent").With(
			"container_id", containerID,
			"endpoint", run.endpoint,
			"runtime", run.runtimeName,
		).Wrapf(err, "start container")
	}
	return streamContainerOutput(run.output, attach), nil
}

func streamContainerOutput(output io.Writer, attach dockertypes.HijackedResponse) <-chan error {
	copyDone := make(chan error, 1)
	go func() {
		defer closeHijackedResponse(attach)
		_, err := stdcopy.StdCopy(output, output, attach.Reader)
		if err != nil && !errors.Is(err, io.EOF) {
			copyDone <- err
			return
		}
		copyDone <- nil
	}()
	return copyDone
}

func (run containerScriptJob) checkContainerStatus(containerID string, status container.WaitResponse) error {
	if status.Error != nil {
		return oops.In("runner_agent").With(
			"container_id", containerID,
			"status_code", status.StatusCode,
			"endpoint", run.endpoint,
			"error", status.Error.Message,
		).New("container execution failed")
	}
	if status.StatusCode != 0 {
		return oops.In("runner_agent").With(
			"container_id", containerID,
			"status_code", status.StatusCode,
			"endpoint", run.endpoint,
		).New("container exited with non-zero status code")
	}
	return nil
}

func cleanupContainer(ctx context.Context, cli *client.Client, containerID string) {
	if containerID == "" {
		return
	}
	removeCtx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()
	if err := cli.ContainerRemove(removeCtx, containerID, container.RemoveOptions{Force: true}); err != nil {
		return
	}
}

func waitContainer(
	ctx context.Context,
	cli *client.Client,
	containerID string,
	copyDone <-chan error,
	endpoint string,
) (container.WaitResponse, error) {
	waitCh, errCh := cli.ContainerWait(ctx, containerID, container.WaitConditionNotRunning)
	select {
	case err := <-errCh:
		if copyErr := readContainerCopy(copyDone, containerID, endpoint); copyErr != nil {
			return container.WaitResponse{}, copyErr
		}
		return container.WaitResponse{}, oops.In("runner_agent").With("container_id", containerID, "endpoint", endpoint).Wrapf(err, "wait container")
	case status := <-waitCh:
		if copyErr := readContainerCopy(copyDone, containerID, endpoint); copyErr != nil {
			return container.WaitResponse{}, copyErr
		}
		return status, nil
	case <-ctx.Done():
		killErr := killContainer(ctx, cli, containerID, endpoint)
		copyErr := readContainerCopy(copyDone, containerID, endpoint)
		if copyErr != nil {
			return container.WaitResponse{}, copyErr
		}
		if killErr != nil {
			return container.WaitResponse{}, killErr
		}
		return container.WaitResponse{}, oops.In("runner_agent").With("container_id", containerID, "endpoint", endpoint).New("container execution canceled")
	}
}

func readContainerCopy(copyDone <-chan error, containerID, endpoint string) error {
	err := <-copyDone
	if err == nil || errors.Is(err, io.EOF) {
		return nil
	}
	return oops.In("runner_agent").With("container_id", containerID, "endpoint", endpoint).Wrapf(err, "read container output")
}

func killContainer(ctx context.Context, cli *client.Client, containerID, endpoint string) error {
	killCtx, cancel := context.WithTimeout(context.WithoutCancel(ctx), 5*time.Second)
	defer cancel()
	if err := cli.ContainerKill(killCtx, containerID, "KILL"); err != nil {
		return oops.In("runner_agent").With("container_id", containerID, "endpoint", endpoint).Wrapf(err, "kill container")
	}
	return nil
}

func closeHijackedResponse(response dockertypes.HijackedResponse) {
	response.Close()
}
