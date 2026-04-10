package gittransport

import (
	"context"
	"io"

	"github.com/DaiYuANg/gity/internal/platform/gitexec"
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

func (s *Service) ReceivePack(ctx context.Context, repoPath string, stdin io.Reader, stdout io.Writer, stderr io.Writer) error {
	return s.runner.Run(ctx, repoPath, []string{"receive-pack", "--stateless-rpc", "."}, stdin, stdout, stderr)
}
