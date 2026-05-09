// Package searchindex wires repository search indexing.
package searchindex

import (
	"context"

	"github.com/arcgolabs/dix"
)

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.searchindex",
		dix.Description("Bleve repository search index refresher"),
		dix.Providers(
			dix.Provider3(NewService),
		),
		dix.Hooks(
			dix.OnStart(func(ctx context.Context, service *Service) error {
				return service.Start(ctx)
			}),
			dix.OnStop(func(ctx context.Context, service *Service) error {
				return service.Stop(ctx)
			}),
		),
	)
}
