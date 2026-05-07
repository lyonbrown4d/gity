package projectissueattachment

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectissueattachment",
		dix.Description("Project issue attachment persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectIssueAttachmentRepository),
		),
	)
}
