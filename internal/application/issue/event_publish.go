package issue

import (
	"context"
	"log/slog"

	domainevent "github.com/lyonbrown4d/gity/internal/domain/event"
	"github.com/samber/oops"
)

func (s *Service) publishIssueEventAsync(ctx context.Context, projectID int64, event domainevent.Event) {
	if s.events == nil {
		return
	}
	if err := s.events.PublishAsync(ctx, event); err != nil {
		wrapped := oops.In("issue").With("project_id", projectID, "event", event.Name()).Wrapf(err, "publish issue event")
		if s.logger != nil {
			s.logger.Warn("publish issue event failed", slog.String("event", event.Name()), slog.String("error", wrapped.Error()))
		}
	}
}
