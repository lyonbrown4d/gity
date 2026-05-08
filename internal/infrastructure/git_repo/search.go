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
	query := strings.TrimSpace(input.Query)
	if query == "" {
		return nil, fmt.Errorf("%w: query is required", ErrInvalidSearchQuery)
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

	limit := normalizeSearchLimit(input.Limit)
	maxFiles := normalizeSearchMaxFiles(input.MaxFiles)
	maxFileSize := normalizeSearchMaxFileSize(input.MaxFileSize)

	matcher, err := buildSearchMatcher(query, input.MatchCase, input.UseRegex)
	if err != nil {
		return nil, err
	}

	pathPrefix := normalizePathPrefix(input.Path)
	results := collectionlist.NewListWithCapacity[SearchResult](limit)
	fileCount := 0

	err = tree.Files().ForEach(func(file *object.File) error {
		if ctx.Err() != nil {
			return ctx.Err()
		}
		if fileCount >= maxFiles {
			return errStopIteration
		}
		if pathPrefix != "" {
			if !isPathInScope(file.Name, pathPrefix) {
				return nil
			}
		}
		if file.Size > maxFileSize {
			return nil
		}
		content, readErr := readBlobContent(file)
		if readErr != nil {
			return oops.In("git_repo").With("repo_path", repoPath, "path", file.Name).Wrapf(readErr, "read searchable blob")
		}
		if !isReadableContent(content) {
			return nil
		}

		for lineNumber, line := range strings.Split(string(content), "\n") {
			column, matchLength, matched := matchLine(line, matcher)
			if !matched {
				continue
			}
			results.Add(SearchResult{
				Path:        file.Name,
				LineNumber:  lineNumber + 1,
				Column:      column,
				MatchLength: matchLength,
				LineContent: line,
			})
			if results.Len() >= limit {
				return errStopIteration
			}
		}

		fileCount++
		return nil
	})
	if err != nil && !errors.Is(err, errStopIteration) {
		return nil, fmt.Errorf("search repository: %w", err)
	}
	return results.Values(), nil
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
