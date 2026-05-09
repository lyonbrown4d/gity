// Package searchindex wires repository search indexing.
package searchindex

import (
	"context"

	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/eventx"
)

func QueryModule() dix.Module {
	return dix.NewModule(
		"infrastructure.searchindex.query",
		dix.Description("Bleve repository search index query adapter"),
		dix.Providers(
			dix.Provider3(NewService),
			dix.Provider1(NewCodeSearchIndex),
		),
	)
}

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.searchindex",
		dix.Description("Bleve repository search index refresher"),
		dix.Providers(
			dix.Provider3(NewService),
			dix.Provider1(NewCodeSearchIndex),
			dix.Provider1(NewSubscriber),
		),
		dix.Hooks(
			dix.OnStart(func(ctx context.Context, service *Service) error {
				return service.Start(ctx)
			}),
			dix.OnStart2(func(_ context.Context, bus eventx.BusRuntime, subscriber *Subscriber) error {
				return subscriber.Subscribe(bus)
			}),
			dix.OnStop(func(_ context.Context, subscriber *Subscriber) error {
				subscriber.Close()
				return nil
			}),
			dix.OnStop(func(ctx context.Context, service *Service) error {
				return service.Stop(ctx)
			}),
		),
	)
}

var _ gitports.CodeSearchIndex = (*Service)(nil)
