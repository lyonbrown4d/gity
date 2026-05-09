// Package projectauditevent wires project audit event persistence.
package projectauditevent

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectauditevent",
		dix.Description("Project audit event persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectAuditEventRepository),
		),
	)
}
