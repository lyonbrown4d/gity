// Package gitsearch contains shared repository code-search matching helpers.
package gitsearch

import (
	"bytes"
	"fmt"
	"regexp"
	"strings"
	"unicode/utf8"

	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	collectionlist "github.com/arcgolabs/collectionx/list"
)

type Plan struct {
	limit       int
	maxFiles    int
	maxFileSize int64
	pathPrefix  string
	matcher     matcher
}

func NewPlan(input gitports.SearchParams) (Plan, error) {
	query := strings.TrimSpace(input.Query)
	if query == "" {
		return Plan{}, fmt.Errorf("%w: query is required", gitports.ErrInvalidSearchQuery)
	}
	searchMatcher, err := buildMatcher(query, input.MatchCase, input.UseRegex)
	if err != nil {
		return Plan{}, err
	}
	return Plan{
		limit:       NormalizeLimit(input.Limit),
		maxFiles:    NormalizeMaxFiles(input.MaxFiles),
		maxFileSize: NormalizeMaxFileSize(input.MaxFileSize),
		pathPrefix:  NormalizePathPrefix(input.Path),
		matcher:     searchMatcher,
	}, nil
}

func (p Plan) Limit() int {
	return p.limit
}

func (p Plan) MaxFiles() int {
	return p.maxFiles
}

func (p Plan) MaxFileSize() int64 {
	return p.maxFileSize
}

func (p Plan) PathPrefix() string {
	return p.pathPrefix
}

func (p Plan) Query() string {
	return p.matcher.raw
}

func AppendMatches(fileName string, content []byte, plan Plan, results *collectionlist.List[gitports.SearchResult]) {
	for lineNumber, line := range strings.Split(string(content), "\n") {
		column, matchLength, matched := matchLine(line, plan.matcher)
		if !matched {
			continue
		}
		results.Add(gitports.SearchResult{Path: fileName, LineNumber: lineNumber + 1, Column: column, MatchLength: matchLength, LineContent: line})
		if results.Len() >= plan.Limit() {
			return
		}
	}
}

func IsPathInScope(filePath, prefix string) bool {
	if filePath == prefix {
		return true
	}
	return strings.HasPrefix(filePath, prefix+"/")
}

func IsReadableContent(data []byte) bool {
	if !utf8.Valid(data) {
		return false
	}
	return !bytes.Contains(data, []byte("\x00"))
}

func NormalizePathPrefix(value string) string {
	return strings.Trim(strings.ReplaceAll(value, "\\", "/"), "/")
}

func NormalizeLimit(value int) int {
	if value <= 0 {
		return 20
	}
	if value > 500 {
		return 500
	}
	return value
}

func NormalizeMaxFiles(value int) int {
	if value <= 0 {
		return 300
	}
	if value > 5000 {
		return 5000
	}
	return value
}

func NormalizeMaxFileSize(value int64) int64 {
	if value <= 0 {
		return 256 * 1024
	}
	if value > 10*1024*1024 {
		return 10 * 1024 * 1024
	}
	return value
}

type matcher struct {
	regex *regexp.Regexp
	query string
	raw   string
}

func buildMatcher(query string, matchCase, useRegex bool) (matcher, error) {
	if !useRegex {
		pattern := query
		if !matchCase {
			pattern = strings.ToLower(pattern)
		}
		return matcher{query: pattern, raw: query}, nil
	}

	expression := query
	if !matchCase && !strings.HasPrefix(expression, "(?i)") {
		expression = "(?i)" + expression
	}
	re, err := regexp.Compile(expression)
	if err != nil {
		return matcher{}, fmt.Errorf("%w: %w", gitports.ErrInvalidSearchRegexp, err)
	}
	return matcher{regex: re, raw: query}, nil
}

func matchLine(line string, matcher matcher) (int, int, bool) {
	if matcher.regex != nil {
		index := matcher.regex.FindStringIndex(line)
		if index == nil {
			return 0, 0, false
		}
		start := utf8.RuneCountInString(line[:index[0]]) + 1
		length := utf8.RuneCountInString(line[index[0]:index[1]])
		return start, length, true
	}
	lineText := line
	target := matcher.query
	if matcher.query != matcher.raw {
		lineText = strings.ToLower(line)
	}
	idx := strings.Index(lineText, target)
	if idx < 0 {
		return 0, 0, false
	}
	end := idx + len(target)
	start := utf8.RuneCountInString(line[:idx]) + 1
	length := utf8.RuneCountInString(line[idx:end])
	return start, length, true
}
