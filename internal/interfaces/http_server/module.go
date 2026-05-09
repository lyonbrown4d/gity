// Package httpserver wires the HTTP server runtime.
package httpserver

import (
	"context"
	"time"

	"github.com/arcgolabs/dix"
)

func Module() dix.Module {
	return dix.NewModule(
		"interfaces.httpserver",
		dix.Description("Fiber HTTP server and lifecycle"),
		dix.Providers(
			dix.Provider0(NewFiberApp, dix.Eager()),
			dix.ProviderErr4(NewServer, dix.Eager()),
			dix.Provider3(NewHost, dix.Eager()),
		),
		dix.Hooks(
			dix.OnStart(func(ctx context.Context, host *Host) error {
				return host.Start(ctx)
			},
				dix.LifecycleName("http_server.start"),
				dix.LifecyclePriority(80),
				dix.LifecycleTimeout(10*time.Second),
			),
			dix.OnStop(func(ctx context.Context, host *Host) error {
				return host.Stop(ctx)
			},
				dix.LifecycleName("http_server.stop"),
				dix.LifecyclePriority(80),
				dix.LifecycleTimeout(15*time.Second),
			),
		),
	)
}
