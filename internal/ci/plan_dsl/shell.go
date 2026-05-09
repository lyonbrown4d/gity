package plandsl

import (
	"regexp"
	"strings"
)

var safeShellArgPattern = regexp.MustCompile(`^[A-Za-z0-9_./:@%+=,-]+$`)

func shellJoin(args []string) string {
	quoted := make([]string, 0, len(args))
	for _, arg := range args {
		quoted = append(quoted, shellQuote(arg))
	}
	return strings.Join(quoted, " ")
}

func shellQuote(value string) string {
	if value != "" && safeShellArgPattern.MatchString(value) {
		return value
	}
	return "'" + strings.ReplaceAll(value, "'", "''") + "'"
}
