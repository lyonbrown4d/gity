package mergerequest

import (
	"path"
	"regexp"
	"strings"
	"unicode"

	setx "github.com/arcgolabs/collectionx/set"
	"github.com/bmatcuk/doublestar/v4"
)

// CodeOwnerRule is one parsed CODEOWNERS pattern with its eligible usernames.
type CodeOwnerRule struct {
	Pattern   string
	Usernames []string
}

var codeOwnerUsernamePattern = regexp.MustCompile(`@([A-Za-z0-9][A-Za-z0-9_.-]*)`)

// ParseCodeOwners parses a CODEOWNERS file into ordered match rules.
func ParseCodeOwners(content string) []CodeOwnerRule {
	rules := make([]CodeOwnerRule, 0)
	for line := range strings.SplitSeq(content, "\n") {
		rule, ok := ParseCodeOwnerLine(line)
		if ok {
			rules = append(rules, rule)
		}
	}
	return rules
}

// ParseCodeOwnerLine parses one CODEOWNERS line.
func ParseCodeOwnerLine(line string) (CodeOwnerRule, bool) {
	line = strings.TrimSpace(StripCodeOwnerComment(line))
	if line == "" || strings.HasPrefix(line, "[") || strings.HasPrefix(line, "!") {
		return CodeOwnerRule{}, false
	}
	fields := SplitCodeOwnerFields(line)
	if len(fields) < 2 {
		return CodeOwnerRule{}, false
	}
	usernames := codeOwnerUsernames(fields[1:])
	if len(usernames) == 0 {
		return CodeOwnerRule{}, false
	}
	return CodeOwnerRule{Pattern: fields[0], Usernames: usernames}, true
}

func codeOwnerUsernames(owners []string) []string {
	usernames := make([]string, 0, len(owners))
	for _, owner := range owners {
		matches := codeOwnerUsernamePattern.FindStringSubmatch(owner)
		if len(matches) == 2 {
			usernames = append(usernames, strings.ToLower(matches[1]))
		}
	}
	return usernames
}

type codeOwnerParserState struct {
	escaped             bool
	prevWasSpaceOrStart bool
}

func (s *codeOwnerParserState) consumeEscapedRune(output *[]rune, value rune) bool {
	if !s.escaped {
		return false
	}
	*output = append(*output, value)
	s.escaped = false
	return true
}

func (s *codeOwnerParserState) consumeEscapePrefix(value rune) bool {
	if value != '\\' {
		return false
	}
	s.escaped = true
	return true
}

func (s *codeOwnerParserState) setSpaceState(value rune) {
	s.prevWasSpaceOrStart = unicode.IsSpace(value)
}

// StripCodeOwnerComment removes unescaped CODEOWNERS comments.
func StripCodeOwnerComment(line string) string {
	line = strings.TrimSpace(line)
	if line == "" {
		return line
	}

	var output []rune
	parser := &codeOwnerParserState{prevWasSpaceOrStart: true}

	for _, r := range line {
		if parser.escaped {
			output = append(output, '\\', r)
			parser.escaped = false
			parser.prevWasSpaceOrStart = false
			continue
		}
		if parser.consumeEscapePrefix(r) {
			continue
		}
		if r == '#' && parser.prevWasSpaceOrStart {
			return string(output)
		}
		output = append(output, r)
		parser.setSpaceState(r)
	}

	if parser.escaped {
		output = append(output, '\\')
	}
	return string(output)
}

// SplitCodeOwnerFields splits a CODEOWNERS line while preserving escaped spaces.
func SplitCodeOwnerFields(line string) []string {
	line = strings.TrimSpace(line)
	if line == "" {
		return nil
	}

	fields := make([]string, 0)
	var field []rune
	parser := &codeOwnerParserState{}

	for _, r := range line {
		if parser.consumeEscapedRune(&field, r) {
			continue
		}
		if parser.consumeEscapePrefix(r) {
			continue
		}
		if unicode.IsSpace(r) {
			flushCodeOwnerField(&field, &fields)
			continue
		}
		field = append(field, r)
	}

	if parser.escaped {
		field = append(field, '\\')
	}
	flushCodeOwnerField(&field, &fields)
	return fields
}

func flushCodeOwnerField(field *[]rune, fields *[]string) {
	if len(*field) == 0 {
		return
	}
	*fields = append(*fields, string(*field))
	*field = (*field)[:0]
}

func matchedCodeOwnerUsernames(rules []CodeOwnerRule, files []string) []string {
	usernames := setx.NewSet[string]()
	for _, file := range files {
		matched := lastMatchingCodeOwnerRule(rules, file)
		if matched == nil {
			continue
		}
		for _, username := range matched.Usernames {
			usernames.Add(username)
		}
	}
	return usernames.Values()
}

func lastMatchingCodeOwnerRule(rules []CodeOwnerRule, file string) *CodeOwnerRule {
	var matched *CodeOwnerRule
	for index := range rules {
		rule := rules[index]
		if codeOwnerPatternMatches(rule.Pattern, file) {
			current := rule
			matched = &current
		}
	}
	return matched
}

func codeOwnerPatternMatches(pattern, filePath string) bool {
	pattern = strings.TrimSpace(strings.TrimPrefix(pattern, "/"))
	filePath = strings.TrimSpace(strings.ReplaceAll(filePath, "\\", "/"))
	if pattern == "" {
		return false
	}
	if pattern == "*" {
		return true
	}
	for _, candidate := range codeOwnerPatternCandidates(pattern) {
		matched, err := doublestar.PathMatch(candidate, filePath)
		if err == nil && matched {
			return true
		}
	}
	return pattern == filePath
}

func codeOwnerPatternCandidates(pattern string) []string {
	candidates := []string{pattern}
	if !strings.Contains(pattern, "/") {
		candidates = append(candidates, path.Join("**", pattern), path.Join("**", pattern+"*"))
	}
	if strings.HasSuffix(pattern, "/") {
		candidates = append(candidates, path.Join(pattern, "**"))
	}
	return candidates
}
