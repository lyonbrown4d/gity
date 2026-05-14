// Package projectcredential wires project credential application services.
package projectcredential

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.project_credential",
		dix.Description("Project credential application services"),
		dix.Providers(
			dix.Provider3(NewService),
		),
	)
}
