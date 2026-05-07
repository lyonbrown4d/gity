package http

import (
	"context"

	"github.com/arcgolabs/dix"
)

func Module() dix.Module {
	return dix.NewModule(
		"http",
		dix.Description("Fiber HTTP server and lifecycle"),
		dix.Providers(
			dix.Provider0(NewFiberApp),
			dix.ProviderErr4(NewServer),
			dix.Provider3(NewHost),
		),
		dix.Hooks(
			dix.OnStart(func(ctx context.Context, host *Host) error {
				return host.Start(ctx)
			}),
			dix.OnStop(func(ctx context.Context, host *Host) error {
				return host.Stop(ctx)
			}),
		),
	)
}
