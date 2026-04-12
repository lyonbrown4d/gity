package issue

import "github.com/DaiYuANg/arcgo/dix"

func Module() dix.Module {
	return dix.NewModule(
		"service.issue",
		dix.Description("Issue application services"),
		dix.Providers(
			dix.Provider6(NewService),
		),
	)
}
