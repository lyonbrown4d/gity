package projectjoblog

import "github.com/arcgolabs/dix"

func Module() dix.Module {
	return dix.NewModule(
		"repository.projectjoblog",
		dix.Description("Project job log persistence"),
		dix.Providers(
			dix.ProviderErr1(NewRepository),
			dix.Provider1(NewProjectJobLogRepository),
		),
	)
}
