// Package gitexec wires native git execution.
package gitexec

import (
	"github.com/arcgolabs/dix"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
)

func NewGitRunner(runner *Runner) gitports.GitRunner {
	return runner
}

func Module() dix.Module {
	return dix.NewModule(
		"infrastructure.gitexec",
		dix.Description("Native git command runner"),
		dix.Providers(
			dix.Provider1(NewRunner),
			dix.Provider1(NewGitRunner),
		),
	)
}
