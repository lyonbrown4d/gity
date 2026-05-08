// Package lfs wires Git LFS HTTP endpoints.
package lfs

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"endpoint.lfs",
		dix.Description("Git LFS routes"),
		dix.Invokes(
			dix.Invoke5(RegisterRoutes),
		),
	)
}
