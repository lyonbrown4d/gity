package plandsl

import (
	"strings"

	"github.com/samber/oops"
)

func commandsToScript(commands []CommandSpec) ([]string, error) {
	script := make([]string, 0, len(commands))
	for _, command := range commands {
		switch command.Name {
		case "shell":
			if len(command.Args) != 1 {
				return nil, oops.In("ci_plan_dsl").With("action", command.Name, "arg_count", len(command.Args)).New("shell expects exactly one argument")
			}
			script = append(script, strings.TrimSpace(command.Args[0]))
		case "exec":
			if len(command.Args) == 0 {
				return nil, oops.In("ci_plan_dsl").With("action", command.Name).New("exec expects at least one argument")
			}
			script = append(script, shellJoin(command.Args))
		default:
			return nil, oops.In("ci_plan_dsl").With("action", command.Name).New("unsupported ci action")
		}
	}
	return script, nil
}
