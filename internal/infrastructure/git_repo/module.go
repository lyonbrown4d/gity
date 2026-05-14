// Package gitrepo wires repository read services.
package gitrepo

import (
	"github.com/arcgolabs/dix"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
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
