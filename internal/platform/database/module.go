package database

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"platform.database",
		dix.Description("Database runtime"),
		dix.Providers(
			dix.ProviderErr2(NewDatabase),
		),
	)
}
