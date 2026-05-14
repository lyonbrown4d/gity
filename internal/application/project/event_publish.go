package project

import (
	"context"
	"log/slog"

	domainevent "github.com/lyonbrown4d/gity/internal/domain/event"
	"github.com/samber/oops"
)

func (s *Service) publishProjectEventAsync(ctx context.Context, projectID int64, event domainevent.Event) {
	if err := s.events.PublishAsync(ctx, event); err != nil {
		wrapped := oops.In("project").With("project_id", projectID, "event", event.Name()).Wrapf(err, "publish project event")
		if s.logger != nil {
			s.logger.Warn("publish project event failed", slog.String("event", event.Name()), slog.String("error", wrapped.Error()))
		}
	}
}
