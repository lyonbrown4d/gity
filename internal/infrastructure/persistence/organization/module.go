// Package organization wires organization persistence.
package organization

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.organization",
		dix.Description("Organization persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewOrganizationRepository),
		),
	)
}
