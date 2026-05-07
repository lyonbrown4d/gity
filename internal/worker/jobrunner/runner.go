package jobrunner

import (
	"context"
	"log/slog"
	"os"
	"strings"
	"sync"
	"time"

	jobservice "github.com/DaiYuANg/gity/internal/application/job"
	"github.com/DaiYuANg/gity/internal/config"
)

type Runner struct {
	logger   *slog.Logger
	service  *jobservice.Service
	settings config.Settings

	mu      sync.Mutex
	cancel  context.CancelFunc
	done    chan struct{}
	started bool
}

func NewRunner(logger *slog.Logger, service *jobservice.Service, settings config.Settings) *Runner {
	if logger == nil {
		logger = slog.Default()
	}
	return &Runner{logger: logger, service: service, settings: settings}
}

func (r *Runner) Start(context.Context) error {
	if !r.settings.Worker.Enabled {
		r.logger.Info("project job runner disabled")
		return nil
	}
	r.mu.Lock()
	defer r.mu.Unlock()
	if r.started {
		return nil
	}
	ctx, cancel := context.WithCancel(context.Background())
	r.cancel = cancel
	r.done = make(chan struct{})
	r.started = true
	go r.loop(ctx)
	r.logger.Info("project job runner started", slog.String("worker_id", r.workerID()))
	return nil
}

func (r *Runner) Stop(ctx context.Context) error {
	r.mu.Lock()
	if !r.started {
		r.mu.Unlock()
		return nil
	}
	cancel := r.cancel
	done := r.done
	r.mu.Unlock()

	cancel()
	select {
	case <-done:
	case <-ctx.Done():
		return ctx.Err()
	}

	r.mu.Lock()
	r.started = false
	r.cancel = nil
	r.done = nil
	r.mu.Unlock()
	r.logger.Info("project job runner stopped")
	return nil
}

func (r *Runner) loop(ctx context.Context) {
	defer close(r.done)
	ticker := time.NewTicker(r.pollInterval())
	defer ticker.Stop()
	for {
		r.drain(ctx)
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
		}
	}
}

func (r *Runner) drain(ctx context.Context) {
	limit := r.settings.Worker.MaxJobsPerTick
	if limit <= 0 {
		limit = 1
	}
	for i := 0; i < limit; i++ {
		claimed, err := r.service.RunNext(ctx, r.workerID(), r.leaseDuration())
		if err != nil {
			r.logger.Error("run project job failed", slog.String("error", err.Error()))
			return
		}
		if !claimed {
			return
		}
	}
}

func (r *Runner) workerID() string {
	configured := strings.TrimSpace(r.settings.Worker.ID)
	if configured != "" {
		return configured
	}
	hostname, err := os.Hostname()
	if err == nil && strings.TrimSpace(hostname) != "" {
		return hostname
	}
	return "worker"
}

func (r *Runner) pollInterval() time.Duration {
	millis := r.settings.Worker.PollIntervalMillis
	if millis <= 0 {
		millis = 1000
	}
	return time.Duration(millis) * time.Millisecond
}

func (r *Runner) leaseDuration() time.Duration {
	seconds := r.settings.Worker.LeaseSeconds
	if seconds <= 0 {
		seconds = 60
	}
	return time.Duration(seconds) * time.Second
}
