package storage

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"platform.storage",
		dix.Description("Attachment storage runtime"),
		dix.Providers(
			dix.ProviderErr1(NewService),
		),
	)
}
