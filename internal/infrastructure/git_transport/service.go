package gittransport

import (
	"context"
	"io"

	"github.com/DaiYuANg/gity/internal/infrastructure/git_exec"
)

type Service struct {
	runner *gitexec.Runner
}

func NewService(runner *gitexec.Runner) *Service {
	return &Service{runner: runner}
}

func (s *Service) UploadPack(ctx context.Context, repoPath string, stdin io.Reader, stdout io.Writer, stderr io.Writer) error {
	return s.runner.Run(ctx, repoPath, []string{"upload-pack", "--stateless-rpc", "."}, stdin, stdout, stderr)
}

func (s *Service) AdvertiseUploadPack(ctx context.Context, repoPath string, stdout io.Writer, stderr io.Writer) error {
	return s.runner.Run(ctx, repoPath, []string{"upload-pack", "--stateless-rpc", "--advertise-refs", "."}, nil, stdout, stderr)
}

func (s *Service) ReceivePack(ctx context.Context, repoPath string, stdin io.Reader, stdout io.Writer, stderr io.Writer) error {
	return s.runner.Run(ctx, repoPath, []string{"receive-pack", "--stateless-rpc", "."}, stdin, stdout, stderr)
}

func (s *Service) AdvertiseReceivePack(ctx context.Context, repoPath string, stdout io.Writer, stderr io.Writer) error {
	return s.runner.Run(ctx, repoPath, []string{"receive-pack", "--stateless-rpc", "--advertise-refs", "."}, nil, stdout, stderr)
}
