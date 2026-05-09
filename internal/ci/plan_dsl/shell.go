package plandsl

import (
	"strings"

	"mvdan.cc/sh/v3/syntax"
)

func shellJoin(args []string) string {
	quoted := make([]string, 0, len(args))
	for _, arg := range args {
		quoted = append(quoted, shellQuote(arg))
	}
	return strings.Join(quoted, " ")
}

func shellQuote(value string) string {
	quoted, err := syntax.Quote(value, syntax.LangBash)
	if err != nil {
		return "''"
	}
	return quoted
}
