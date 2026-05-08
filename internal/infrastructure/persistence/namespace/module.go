// Package namespace wires namespace persistence.
package namespace

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.namespace",
		dix.Description("Namespace persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewNamespaceRepository),
		),
	)
}
