package eventbus

import (
	"context"

	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/eventx"
)

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.eventbus",
		dix.Description("Domain event bus runtime"),
		dix.Providers(
			dix.Provider1(NewBus),
			dix.Provider1(NewPublisher),
		),
		dix.Hooks(
			dix.OnStop(func(_ context.Context, bus eventx.BusRuntime) error {
				if bus == nil {
					return nil
				}
				return bus.Close()
			}),
		),
	)
}
