// Package config wires configuration loading.
package config

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"config",
		dix.Description("Configuration loading"),
		dix.Providers(
			dix.ProviderErr0(NewConfig),
			dix.ProviderErr1(NewSettings),
		),
	)
}
