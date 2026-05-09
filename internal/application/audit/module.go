// Package audit wires audit event consumers.
package audit

import (
	"context"

	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/eventx"
)

func Module() dix.Module {
	return dix.NewModule(
		"service.audit",
		dix.Description("Audit event consumers"),
		dix.Providers(
			dix.Provider2(NewService),
			dix.Provider2(NewSubscriber),
		),
		dix.Hooks(
			dix.OnStart2(func(_ context.Context, bus eventx.BusRuntime, subscriber *Subscriber) error {
				return subscriber.Subscribe(bus)
			}),
			dix.OnStop(func(_ context.Context, subscriber *Subscriber) error {
				subscriber.Close()
				return nil
			}),
		),
	)
}
