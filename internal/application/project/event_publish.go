package project

import (
	"context"
	"log/slog"

	domainevent "github.com/DaiYuANg/gity/internal/domain/event"
	"github.com/samber/oops"
)

func (s *Service) publishProjectEventAsync(ctx context.Context, projectID int64, event domainevent.Event) {
	if err := s.events.PublishAsync(ctx, event); err != nil {
		wrapped := oops.In("project").With("project_id", projectID, "event", event.Name()).Wrapf(err, "publish project event")
		logger := s.logger
		if logger == nil {
			logger = slog.Default()
		}
		logger.Warn("publish project event failed", slog.String("event", event.Name()), slog.String("error", wrapped.Error()))
	}
}
