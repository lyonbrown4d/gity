// Package jobrunner wires background project job runners.
package jobrunner

import (
	"context"
	"time"

	"github.com/arcgolabs/dix"
)

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.worker.jobrunner",
		dix.Description("Project job worker runtime"),
		dix.Providers(
			dix.Provider3(NewRunner, dix.Eager()),
		),
		dix.Hooks(
			dix.OnStart(func(ctx context.Context, runner *Runner) error {
				return runner.Start(ctx)
			},
				dix.LifecycleName("job_runner.start"),
				dix.LifecyclePriority(70),
				dix.LifecycleTimeout(10*time.Second),
			),
			dix.OnStop(func(ctx context.Context, runner *Runner) error {
				return runner.Stop(ctx)
			},
				dix.LifecycleName("job_runner.stop"),
				dix.LifecyclePriority(70),
				dix.LifecycleTimeout(15*time.Second),
			),
		),
	)
}
