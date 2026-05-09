package logger

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.logger",
		dix.Description("Structured logging"),
		dix.Providers(
			dix.ProviderErr1(NewLogger, dix.Eager()),
		),
	)
}
