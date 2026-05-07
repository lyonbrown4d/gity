package eventbus

import (
	"context"
	"log/slog"

	appports "github.com/DaiYuANg/gity/internal/application/ports"
	domainevent "github.com/DaiYuANg/gity/internal/domain/event"
	"github.com/arcgolabs/eventx"
)

type Publisher struct {
	bus eventx.BusRuntime
}

func NewBus(logger *slog.Logger) eventx.BusRuntime {
	if logger == nil {
		logger = slog.Default()
	}
	return eventx.New(
		eventx.WithParallelDispatch(false),
		eventx.WithMiddleware(eventx.RecoverMiddleware()),
		eventx.WithAsyncErrorHandler(func(_ context.Context, event eventx.Event, err error) {
			logger.Error("domain event handler failed", slog.String("event", event.Name()), slog.String("error", err.Error()))
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
	return p.bus.Publish(ctx, event)
}

func (p Publisher) PublishAsync(ctx context.Context, event domainevent.Event) error {
	if p.bus == nil || event == nil {
		return nil
	}
	return p.bus.PublishAsync(ctx, event)
}
