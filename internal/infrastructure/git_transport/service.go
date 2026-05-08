package gittransport

import (
	"context"
	"io"

	"github.com/DaiYuANg/gity/internal/infrastructure/git_exec"
	"github.com/samber/oops"
)

type Service struct {
	runner *gitexec.Runner
}

func NewService(runner *gitexec.Runner) *Service {
	return &Service{runner: runner}
}

func (s *Service) UploadPack(ctx context.Context, repoPath string, stdin io.Reader, stdout, stderr io.Writer) error {
	if err := s.runner.Run(ctx, repoPath, []string{"upload-pack", "--stateless-rpc", "."}, stdin, stdout, stderr); err != nil {
		return oops.In("git_transport").With("repo_path", repoPath).Wrapf(err, "run upload-pack")
	}
	return nil
}

func (s *Service) AdvertiseUploadPack(ctx context.Context, repoPath string, stdout, stderr io.Writer) error {
	if err := s.runner.Run(ctx, repoPath, []string{"upload-pack", "--stateless-rpc", "--advertise-refs", "."}, nil, stdout, stderr); err != nil {
		return oops.In("git_transport").With("repo_path", repoPath).Wrapf(err, "advertise upload-pack")
	}
	return nil
}

func (s *Service) ReceivePack(ctx context.Context, repoPath string, stdin io.Reader, stdout, stderr io.Writer) error {
	if err := s.runner.Run(ctx, repoPath, []string{"receive-pack", "--stateless-rpc", "."}, stdin, stdout, stderr); err != nil {
		return oops.In("git_transport").With("repo_path", repoPath).Wrapf(err, "run receive-pack")
	}
	return nil
}

func (s *Service) AdvertiseReceivePack(ctx context.Context, repoPath string, stdout, stderr io.Writer) error {
	if err := s.runner.Run(ctx, repoPath, []string{"receive-pack", "--stateless-rpc", "--advertise-refs", "."}, nil, stdout, stderr); err != nil {
		return oops.In("git_transport").With("repo_path", repoPath).Wrapf(err, "advertise receive-pack")
	}
	return nil
}
