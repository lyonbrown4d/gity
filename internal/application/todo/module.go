package todo

import (
	"context"
	"time"

	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/eventx"
)

func Module() dix.Module {
	return dix.NewModule(
		"service.todo",
		dix.Description("Todo application services and event consumers"),
		dix.Providers(
			dix.Provider1(NewService),
			dix.Provider4(NewSubscriber),
		),
		dix.Hooks(
			dix.OnStart2(func(_ context.Context, bus *eventx.Bus, subscriber *Subscriber) error {
				return subscriber.Subscribe(bus)
			},
				dix.LifecycleName("todo.subscribe"),
				dix.LifecyclePriority(35),
				dix.LifecycleParallel(),
				dix.LifecycleTimeout(5*time.Second),
			),
			dix.OnStop(func(_ context.Context, subscriber *Subscriber) error {
				subscriber.Close()
				return nil
			},
				dix.LifecycleName("todo.unsubscribe"),
				dix.LifecyclePriority(35),
				dix.LifecycleParallel(),
				dix.LifecycleTimeout(5*time.Second),
			),
		),
	)
}
