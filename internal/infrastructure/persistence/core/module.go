// Package core wires persistence schema bootstrap.
package core

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.core",
		dix.Description("Schema bootstrap"),
		dix.Hooks(
			dix.OnStart(EnsureSchema),
		),
	)
}
