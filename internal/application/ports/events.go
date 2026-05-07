package ports

import (
	"context"

	domainevent "github.com/DaiYuANg/gity/internal/domain/event"
)

type DomainEventPublisher interface {
	Publish(ctx context.Context, event domainevent.Event) error
	PublishAsync(ctx context.Context, event domainevent.Event) error
}

type NoopDomainEventPublisher struct{}

func (NoopDomainEventPublisher) Publish(context.Context, domainevent.Event) error {
	return nil
}

func (NoopDomainEventPublisher) PublishAsync(context.Context, domainevent.Event) error {
	return nil
}
