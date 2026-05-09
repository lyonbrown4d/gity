package searchindex

import (
	"context"

	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	"github.com/arcgolabs/eventx"
	"github.com/samber/oops"
)

type Subscriber struct {
	service     *Service
	unsubscribe []func()
}

func NewSubscriber(service *Service) *Subscriber {
	return &Subscriber{service: service}
}

func (s *Subscriber) Subscribe(bus eventx.BusRuntime) error {
	if bus == nil || s.service == nil {
		return nil
	}
	return oops.Join(
		s.subscribeProjectCreated(bus),
		s.subscribeProjectDeleted(bus),
		s.subscribeProjectRepositoryChanged(bus),
	)
}

func (s *Subscriber) Close() {
	for _, unsubscribe := range s.unsubscribe {
		if unsubscribe != nil {
			unsubscribe()
		}
	}
	s.unsubscribe = nil
}

func (s *Subscriber) subscribeProjectCreated(bus eventx.BusRuntime) error {
	unsubscribe, err := eventx.Subscribe(bus, s.handleProjectCreated)
	if err != nil {
		return oops.In("search_index").Wrapf(err, "subscribe project created search indexing")
	}
	s.unsubscribe = append(s.unsubscribe, unsubscribe)
	return nil
}

func (s *Subscriber) subscribeProjectDeleted(bus eventx.BusRuntime) error {
	unsubscribe, err := eventx.Subscribe(bus, s.handleProjectDeleted)
	if err != nil {
		return oops.In("search_index").Wrapf(err, "subscribe project deleted search indexing")
	}
	s.unsubscribe = append(s.unsubscribe, unsubscribe)
	return nil
}

func (s *Subscriber) subscribeProjectRepositoryChanged(bus eventx.BusRuntime) error {
	unsubscribe, err := eventx.Subscribe(bus, s.handleProjectRepositoryChanged)
	if err != nil {
		return oops.In("search_index").Wrapf(err, "subscribe project repository changed search indexing")
	}
	s.unsubscribe = append(s.unsubscribe, unsubscribe)
	return nil
}

func (s *Subscriber) handleProjectCreated(ctx context.Context, event projectdomain.ProjectCreated) error {
	return s.service.RefreshProject(ctx, projectdomain.Project{
		ID:            event.ProjectID,
		FullPath:      event.FullPath,
		DefaultBranch: event.DefaultBranch,
	})
}

func (s *Subscriber) handleProjectDeleted(ctx context.Context, event projectdomain.ProjectDeleted) error {
	return s.service.DeleteProject(ctx, event.ProjectID)
}

func (s *Subscriber) handleProjectRepositoryChanged(ctx context.Context, event projectdomain.ProjectRepositoryChanged) error {
	if !event.AffectsDefaultBranch() {
		return nil
	}
	return s.service.RefreshProject(ctx, projectdomain.Project{
		ID:            event.ProjectID,
		FullPath:      event.FullPath,
		DefaultBranch: event.DefaultBranch,
	})
}
