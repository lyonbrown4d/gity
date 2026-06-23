// Package audit wires audit event consumers.
package audit

import (
	"context"
	"time"

	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/eventx"
)

func Module() dix.Module {
	return dix.NewModule(
		"service.audit",
		dix.Description("Audit event consumers"),
		dix.Providers(
			dix.Provider2(NewDependencies),
			dix.Provider1(NewServiceWithDependencies),
			dix.Provider2(NewSubscriber),
		),
		dix.Hooks(
			dix.OnStart2(func(_ context.Context, bus eventx.BusRuntime, subscriber *Subscriber) error {
				return subscriber.Subscribe(bus)
			},
				dix.LifecycleName("audit.subscribe"),
				dix.LifecyclePriority(30),
				dix.LifecycleParallel(),
				dix.LifecycleTimeout(5*time.Second),
			),
			dix.OnStop(func(_ context.Context, subscriber *Subscriber) error {
				subscriber.Close()
				return nil
			},
				dix.LifecycleName("audit.unsubscribe"),
				dix.LifecyclePriority(30),
				dix.LifecycleParallel(),
				dix.LifecycleTimeout(5*time.Second),
			),
		),
	)
}
