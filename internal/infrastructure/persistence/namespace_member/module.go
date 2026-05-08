// Package namespacemember wires namespace member persistence.
package namespacemember

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.namespacemember",
		dix.Description("Namespace member persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewNamespaceMemberRepository),
		),
	)
}
