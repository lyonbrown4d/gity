package jobrunner

import (
	"context"

	"github.com/arcgolabs/dix"
)

func Module() dix.Module {
	return dix.NewModule(
		"worker.jobrunner",
		dix.Description("Project job worker runtime"),
		dix.Providers(
			dix.Provider3(NewRunner),
		),
		dix.Hooks(
			dix.OnStart(func(ctx context.Context, runner *Runner) error {
				return runner.Start(ctx)
			}),
			dix.OnStop(func(ctx context.Context, runner *Runner) error {
				return runner.Stop(ctx)
			}),
		),
	)
}
