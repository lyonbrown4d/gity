// Package eventbus implements application event dispatch.
package eventbus

import (
	"context"
	"log/slog"

	"github.com/arcgolabs/eventx"
	appports "github.com/lyonbrown4d/gity/internal/application/ports"
	domainevent "github.com/lyonbrown4d/gity/internal/domain/event"
	"github.com/samber/oops"
)

type Publisher struct {
	bus eventx.BusRuntime
}

func NewBus(logger *slog.Logger) eventx.BusRuntime {
	return eventx.New(
		eventx.WithParallelDispatch(false),
		eventx.WithMiddleware(eventx.RecoverMiddleware()),
		eventx.WithAsyncErrorHandler(func(_ context.Context, event eventx.Event, err error) {
			if logger != nil {
				logger.Error("domain event handler failed", slog.String("event", event.Name()), slog.String("error", err.Error()))
			}
		}),
	)
}

func NewPublisher(bus eventx.BusRuntime) appports.DomainEventPublisher {
	return Publisher{bus: bus}
}

func (p Publisher) Publish(ctx context.Context, event domainevent.Event) error {
	if p.bus == nil || event == nil {
		return nil
	}
	if err := p.bus.Publish(ctx, event); err != nil {
		return oops.In("event_bus").With("event", event.Name()).Wrapf(err, "publish domain event")
	}
	return nil
}

func (p Publisher) PublishAsync(ctx context.Context, event domainevent.Event) error {
	if p.bus == nil || event == nil {
		return nil
	}
	if err := p.bus.PublishAsync(ctx, event); err != nil {
		return oops.In("event_bus").With("event", event.Name()).Wrapf(err, "publish domain event async")
	}
	return nil
}
