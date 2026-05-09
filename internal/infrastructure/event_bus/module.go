package eventbus

import (
	"context"
	"time"

	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/eventx"
)

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.eventbus",
		dix.Description("Domain event bus runtime"),
		dix.Providers(
			dix.Provider1(NewBus, dix.Eager()),
			dix.Provider1(NewPublisher, dix.Eager()),
		),
		dix.Hooks(
			dix.OnStop(func(_ context.Context, bus eventx.BusRuntime) error {
				if bus == nil {
					return nil
				}
				return bus.Close()
			},
				dix.LifecycleName("eventbus.close"),
				dix.LifecyclePriority(20),
				dix.LifecycleTimeout(5*time.Second),
			),
		),
	)
}
