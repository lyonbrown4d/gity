// Package organizationmember wires organization member persistence.
package organizationmember

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.organizationmember",
		dix.Description("Organization member persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewOrganizationMemberRepository),
		),
	)
}
