// Package gitrepo wires repository read services.
package gitrepo

import (
	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	"github.com/arcgolabs/dix"
)

func NewGitRepository(service *Service) gitports.GitRepository {
	return service
}

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.gitrepo",
		dix.Description("go-git repository read services"),
		dix.Providers(
			dix.Provider1(NewService),
			dix.Provider1(NewGitRepository),
		),
	)
}
