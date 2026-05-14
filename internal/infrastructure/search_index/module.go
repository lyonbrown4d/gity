// Package searchindex wires repository search indexing.
package searchindex

import (
	"context"
	"time"

	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/eventx"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
)

func QueryModule() dix.Module {
	return dix.NewModule(
		"infrastructure.searchindex.query",
		dix.Description("Bleve repository search index query adapter"),
		dix.Providers(
			dix.Provider3(NewService, dix.Eager()),
			dix.Provider1(NewCodeSearchIndex, dix.Eager()),
		),
	)
}

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.searchindex",
		dix.Description("Bleve repository search index refresher"),
		dix.Providers(
			dix.Provider3(NewService, dix.Eager()),
			dix.Provider1(NewCodeSearchIndex, dix.Eager()),
			dix.Provider1(NewSubscriber),
		),
		dix.Hooks(
			dix.OnStart(func(ctx context.Context, service *Service) error {
				return service.Start(ctx)
			},
				dix.LifecycleName("search_index.start"),
				dix.LifecyclePriority(40),
				dix.LifecycleTimeout(10*time.Second),
			),
			dix.OnStart2(func(_ context.Context, bus eventx.BusRuntime, subscriber *Subscriber) error {
				return subscriber.Subscribe(bus)
			},
				dix.LifecycleName("search_index.subscribe"),
				dix.LifecyclePriority(30),
				dix.LifecycleParallel(),
				dix.LifecycleTimeout(5*time.Second),
			),
			dix.OnStop(func(_ context.Context, subscriber *Subscriber) error {
				subscriber.Close()
				return nil
			},
				dix.LifecycleName("search_index.unsubscribe"),
				dix.LifecyclePriority(30),
				dix.LifecycleParallel(),
				dix.LifecycleTimeout(5*time.Second),
			),
			dix.OnStop(func(ctx context.Context, service *Service) error {
				return service.Stop(ctx)
			},
				dix.LifecycleName("search_index.stop"),
				dix.LifecyclePriority(40),
				dix.LifecycleTimeout(15*time.Second),
			),
		),
	)
}

var _ gitports.CodeSearchIndex = (*Service)(nil)
