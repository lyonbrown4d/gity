package gitrepo

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"io"
	"regexp"
	"strings"
	"unicode/utf8"

	collectionlist "github.com/arcgolabs/collectionx/list"
	"github.com/go-git/go-git/v5/plumbing/object"
	"github.com/samber/oops"
)

func (s *Service) Search(ctx context.Context, repoPath, refName, defaultBranch string, input SearchParams) ([]SearchResult, error) {
	plan, err := newSearchPlan(input)
	if err != nil {
		return nil, err
	}
	repository, err := s.openRepository(repoPath)
	if err != nil {
		return nil, err
	}

	commit, err := s.resolveCommit(ctx, repository, refName, defaultBranch)
	if err != nil {
		if errors.Is(err, ErrEmptyRepository) {
			return []SearchResult{}, nil
		}
		return nil, err
	}

	tree, err := commit.Tree()
	if err != nil {
		return nil, fmt.Errorf("load commit tree: %w", err)
	}

	return s.searchTree(ctx, repoPath, tree, plan)
}

type searchPlan struct {
	limit       int
	maxFiles    int
	maxFileSize int64
	pathPrefix  string
	matcher     searchMatcher
}

func newSearchPlan(input SearchParams) (searchPlan, error) {
	query := strings.TrimSpace(input.Query)
	if query == "" {
		return searchPlan{}, fmt.Errorf("%w: query is required", ErrInvalidSearchQuery)
	}
	matcher, err := buildSearchMatcher(query, input.MatchCase, input.UseRegex)
	if err != nil {
		return searchPlan{}, err
	}
	return searchPlan{
		limit:       normalizeSearchLimit(input.Limit),
		maxFiles:    normalizeSearchMaxFiles(input.MaxFiles),
		maxFileSize: normalizeSearchMaxFileSize(input.MaxFileSize),
		pathPrefix:  normalizePathPrefix(input.Path),
		matcher:     matcher,
	}, nil
}

func (s *Service) searchTree(ctx context.Context, repoPath string, tree *object.Tree, plan searchPlan) ([]SearchResult, error) {
	results := collectionlist.NewListWithCapacity[SearchResult](plan.limit)
	state := searchTreeState{}
	err := tree.Files().ForEach(func(file *object.File) error {
		return s.searchTreeFile(ctx, repoPath, file, plan, &state, results)
	})
	if err != nil && !errors.Is(err, errStopIteration) {
		return nil, fmt.Errorf("search repository: %w", err)
	}
	return results.Values(), nil
}

type searchTreeState struct {
	fileCount int
}

func (s *Service) searchTreeFile(ctx context.Context, repoPath string, file *object.File, plan searchPlan, state *searchTreeState, results *collectionlist.List[SearchResult]) error {
	if ctx.Err() != nil {
		return oops.In("git_repo").With("repo_path", repoPath).Wrapf(ctx.Err(), "search repository canceled")
	}
	if state.fileCount >= plan.maxFiles {
		return errStopIteration
	}
	searched, err := s.searchFile(repoPath, file, plan, results)
	if err != nil {
		return err
	}
	if searched {
		state.fileCount++
	}
	if results.Len() >= plan.limit {
		return errStopIteration
	}
	return nil
}

func (s *Service) searchFile(repoPath string, file *object.File, plan searchPlan, results *collectionlist.List[SearchResult]) (bool, error) {
	if plan.pathPrefix != "" && !isPathInScope(file.Name, plan.pathPrefix) {
		return false, nil
	}
	if file.Size > plan.maxFileSize {
		return false, nil
	}
	content, readErr := readBlobContent(file)
	if readErr != nil {
		return false, oops.In("git_repo").With("repo_path", repoPath, "path", file.Name).Wrapf(readErr, "read searchable blob")
	}
	if !isReadableContent(content) {
		return true, nil
	}
	appendSearchMatches(file.Name, content, plan.matcher, results, plan.limit)
	return true, nil
}

func appendSearchMatches(fileName string, content []byte, matcher searchMatcher, results *collectionlist.List[SearchResult], limit int) {
	for lineNumber, line := range strings.Split(string(content), "\n") {
		column, matchLength, matched := matchLine(line, matcher)
		if !matched {
			continue
		}
		results.Add(SearchResult{Path: fileName, LineNumber: lineNumber + 1, Column: column, MatchLength: matchLength, LineContent: line})
		if results.Len() >= limit {
			return
		}
	}
}

type searchMatcher struct {
	regex *regexp.Regexp
	query string
	raw   string
}

func buildSearchMatcher(query string, matchCase, useRegex bool) (searchMatcher, error) {
	if !useRegex {
		pattern := query
		if !matchCase {
			pattern = strings.ToLower(pattern)
		}
		return searchMatcher{query: pattern, raw: query}, nil
	}

	expression := query
	if !matchCase && !strings.HasPrefix(expression, "(?i)") {
		expression = "(?i)" + expression
	}
	re, err := regexp.Compile(expression)
	if err != nil {
		return searchMatcher{}, fmt.Errorf("%w: %w", ErrInvalidSearchRegexp, err)
	}
	return searchMatcher{regex: re, raw: query}, nil
}

func matchLine(line string, matcher searchMatcher) (int, int, bool) {
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

func isPathInScope(filePath, prefix string) bool {
	if filePath == prefix {
		return true
	}
	return strings.HasPrefix(filePath, prefix+"/")
}

func readBlobContent(file *object.File) (content []byte, err error) {
	reader, err := file.Reader()
	if err != nil {
		return nil, oops.In("git_repo").With("path", file.Name).Wrapf(err, "open blob reader")
	}
	defer func() {
		if closeErr := reader.Close(); closeErr != nil {
			if err != nil {
				err = oops.In("git_repo").With("path", file.Name).Wrapf(oops.Join(err, closeErr), "read blob content and close reader")
				return
			}
			err = oops.In("git_repo").With("path", file.Name).Wrapf(closeErr, "close blob reader")
		}
	}()
	content, err = io.ReadAll(reader)
	if err != nil {
		return nil, oops.In("git_repo").With("path", file.Name).Wrapf(err, "read blob content")
	}
	return content, nil
}

func normalizePathPrefix(value string) string {
	return strings.Trim(strings.ReplaceAll(value, "\\", "/"), "/")
}

func normalizeSearchLimit(value int) int {
	if value <= 0 {
		return 20
	}
	if value > 500 {
		return 500
	}
	return value
}

func normalizeSearchMaxFiles(value int) int {
	if value <= 0 {
		return 300
	}
	if value > 5000 {
		return 5000
	}
	return value
}

func normalizeSearchMaxFileSize(value int64) int64 {
	if value <= 0 {
		return 256 * 1024
	}
	if value > 10*1024*1024 {
		return 10 * 1024 * 1024
	}
	return value
}

func isReadableContent(data []byte) bool {
	if !utf8.Valid(data) {
		return false
	}
	return !bytes.Contains(data, []byte("\x00"))
}
