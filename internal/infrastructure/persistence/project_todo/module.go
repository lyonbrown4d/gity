// Package projecttodo wires project todo persistence.
package projecttodo

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projecttodo",
		dix.Description("Project todo persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectTodoRepository),
		),
	)
}
