// Package core wires persistence schema bootstrap.
package core

import (
	"time"

	"github.com/arcgolabs/dix"
)

func Module() dix.Module {
	return dix.NewModule(
		"repository.core",
		dix.Description("Schema bootstrap"),
		dix.Hooks(
			dix.OnStart(EnsureSchema,
				dix.LifecycleName("schema.ensure"),
				dix.LifecyclePriority(20),
				dix.LifecycleTimeout(30*time.Second),
			),
		),
	)
}
