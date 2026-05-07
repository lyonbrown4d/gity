package mapperx

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.mapper",
		dix.Description("Shared object mapper"),
		dix.Providers(
			dix.Provider0(NewMapper),
		),
	)
}
