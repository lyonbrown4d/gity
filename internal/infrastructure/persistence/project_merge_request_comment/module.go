// Package projectmergerequestcomment wires project merge request comment persistence.
package projectmergerequestcomment

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectmergerequestcomment",
		dix.Description("Project merge request comment persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectMergeRequestCommentRepository),
		),
	)
}
