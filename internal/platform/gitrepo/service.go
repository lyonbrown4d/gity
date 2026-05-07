package gitrepo

import (
	"bytes"
	"context"
	"encoding/base64"
	"errors"
	"fmt"
	"io"
	"os"
	"path"
	"path/filepath"
	"regexp"
	"slices"
	"strings"
	"unicode/utf8"

	"github.com/DaiYuANg/gity/internal/config"
	setx "github.com/arcgolabs/collectionx/set"
	git "github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git/go-git/v5/plumbing/filemode"
	"github.com/go-git/go-git/v5/plumbing/object"
)

var (
	ErrRepositoryNotFound  = errors.New("git repository not found")
	ErrReferenceNotFound   = errors.New("git reference not found")
	ErrPathNotFound        = errors.New("git path not found")
	ErrReadmeNotFound      = errors.New("git readme not found")
	ErrEmptyRepository     = errors.New("git repository is empty")
	ErrInvalidSearchQuery  = errors.New("invalid search query")
	ErrInvalidSearchRegexp = errors.New("invalid search regex")
	errStopIteration       = errors.New("stop iteration")
)

type Service struct {
	repoRoot string
}

type Branch struct {
	Name      string `json:"name"`
	Hash      string `json:"hash"`
	IsDefault bool   `json:"is_default"`
}

type TreeEntry struct {
	Name string `json:"name"`
	Path string `json:"path"`
	Type string `json:"type"`
	Mode string `json:"mode"`
	Size int64  `json:"size"`
}

type Blob struct {
	Name     string `json:"name"`
	Path     string `json:"path"`
	Encoding string `json:"encoding"`
	Size     int64  `json:"size"`
	Content  string `json:"content"`
}

type Commit struct {
	Hash        string `json:"hash"`
	ShortHash   string `json:"short_hash"`
	AuthorName  string `json:"author_name"`
	AuthorEmail string `json:"author_email"`
	Message     string `json:"message"`
	CommittedAt string `json:"committed_at"`
}

type SearchResult struct {
	Path        string `json:"path"`
	LineNumber  int    `json:"line_number"`
	Column      int    `json:"column"`
	MatchLength int    `json:"match_length"`
	LineContent string `json:"line_content"`
}

type LanguageStat struct {
	Language   string  `json:"language"`
	Bytes      int64   `json:"bytes"`
	Percentage float64 `json:"percentage"`
}

type LanguageAnalysis struct {
	Revision   string         `json:"revision"`
	TotalBytes int64          `json:"total_bytes"`
	Languages  []LanguageStat `json:"languages"`
}

type SearchParams struct {
	Query       string `json:"query"`
	Path        string `json:"path"`
	Limit       int    `json:"limit"`
	MaxFiles    int    `json:"max_files"`
	MaxFileSize int64  `json:"max_file_size"`
	MatchCase   bool   `json:"match_case"`
	UseRegex    bool   `json:"regex"`
}

func NewService(settings config.Settings) *Service {
	return &Service{repoRoot: settings.Git.RepoRoot}
}

func (s *Service) ListBranches(ctx context.Context, repoPath string, defaultBranch string) ([]Branch, error) {
	repository, err := s.openRepository(repoPath)
	if err != nil {
		return nil, err
	}
	iter, err := repository.Branches()
	if err != nil {
		if errors.Is(err, plumbing.ErrReferenceNotFound) {
			return []Branch{}, nil
		}
		return nil, fmt.Errorf("list branches: %w", err)
	}
	defer iter.Close()

	branches := make([]Branch, 0)
	err = iter.ForEach(func(ref *plumbing.Reference) error {
		if ctx.Err() != nil {
			return ctx.Err()
		}
		branches = append(branches, Branch{
			Name:      ref.Name().Short(),
			Hash:      ref.Hash().String(),
			IsDefault: ref.Name().Short() == strings.TrimSpace(defaultBranch),
		})
		return nil
	})
	if err != nil {
		return nil, fmt.Errorf("iterate branches: %w", err)
	}
	slices.SortFunc(branches, func(a Branch, b Branch) int {
		if a.IsDefault && !b.IsDefault {
			return -1
		}
		if !a.IsDefault && b.IsDefault {
			return 1
		}
		return strings.Compare(a.Name, b.Name)
	})
	return branches, nil
}

func (s *Service) ListTree(ctx context.Context, repoPath string, refName string, defaultBranch string, treePath string) ([]TreeEntry, error) {
	repository, err := s.openRepository(repoPath)
	if err != nil {
		return nil, err
	}
	tree, err := s.resolveTree(ctx, repository, refName, defaultBranch, treePath)
	if err != nil {
		if errors.Is(err, ErrEmptyRepository) {
			return []TreeEntry{}, nil
		}
		return nil, err
	}

	entries := make([]TreeEntry, 0, len(tree.Entries))
	for _, entry := range tree.Entries {
		size := int64(0)
		entryType := "blob"
		if entry.Mode == filemode.Dir {
			entryType = "tree"
		} else {
			file, fileErr := tree.File(entry.Name)
			if fileErr == nil {
				size = file.Blob.Size
			}
		}
		entryPath := entry.Name
		if strings.TrimSpace(treePath) != "" {
			entryPath = path.Join(strings.Trim(strings.ReplaceAll(treePath, "\\", "/"), "/"), entry.Name)
		}
		entries = append(entries, TreeEntry{
			Name: entry.Name,
			Path: entryPath,
			Type: entryType,
			Mode: entry.Mode.String(),
			Size: size,
		})
	}
	slices.SortFunc(entries, func(a TreeEntry, b TreeEntry) int {
		if a.Type != b.Type {
			if a.Type == "tree" {
				return -1
			}
			return 1
		}
		return strings.Compare(a.Name, b.Name)
	})
	return entries, nil
}

func (s *Service) GetBlob(ctx context.Context, repoPath string, refName string, defaultBranch string, blobPath string) (Blob, error) {
	repository, err := s.openRepository(repoPath)
	if err != nil {
		return Blob{}, err
	}
	if strings.TrimSpace(blobPath) == "" {
		return Blob{}, fmt.Errorf("%w: blob path is required", ErrPathNotFound)
	}
	normalizedPath := strings.Trim(strings.ReplaceAll(blobPath, "\\", "/"), "/")
	tree, err := s.resolveTree(ctx, repository, refName, defaultBranch, "")
	if err != nil {
		return Blob{}, err
	}
	file, err := tree.File(normalizedPath)
	if err != nil {
		if errors.Is(err, object.ErrFileNotFound) {
			return Blob{}, fmt.Errorf("%w: %s", ErrPathNotFound, normalizedPath)
		}
		return Blob{}, fmt.Errorf("read blob %s: %w", normalizedPath, err)
	}
	return buildBlob(file)
}

func (s *Service) GetReadme(ctx context.Context, repoPath string, refName string, defaultBranch string) (Blob, error) {
	repository, err := s.openRepository(repoPath)
	if err != nil {
		return Blob{}, err
	}
	tree, err := s.resolveTree(ctx, repository, refName, defaultBranch, "")
	if err != nil {
		return Blob{}, err
	}
	for _, entry := range tree.Entries {
		if entry.Mode == filemode.Dir {
			continue
		}
		if isReadmeName(entry.Name) {
			file, fileErr := tree.File(entry.Name)
			if fileErr != nil {
				return Blob{}, fmt.Errorf("read readme %s: %w", entry.Name, fileErr)
			}
			return buildBlob(file)
		}
	}
	return Blob{}, ErrReadmeNotFound
}

func (s *Service) ListCommits(ctx context.Context, repoPath string, refName string, defaultBranch string, limit int) ([]Commit, error) {
	repository, err := s.openRepository(repoPath)
	if err != nil {
		return nil, err
	}
	commit, err := s.resolveCommit(ctx, repository, refName, defaultBranch)
	if err != nil {
		if errors.Is(err, ErrEmptyRepository) {
			return []Commit{}, nil
		}
		return nil, err
	}
	if limit <= 0 {
		limit = 20
	}
	if limit > 100 {
		limit = 100
	}
	iter, err := repository.Log(&git.LogOptions{From: commit.Hash})
	if err != nil {
		return nil, fmt.Errorf("list commits: %w", err)
	}
	defer iter.Close()

	commits := make([]Commit, 0, limit)
	count := 0
	err = iter.ForEach(func(item *object.Commit) error {
		if ctx.Err() != nil {
			return ctx.Err()
		}
		if count >= limit {
			return errStopIteration
		}
		hash := item.Hash.String()
		shortHash := hash
		if len(shortHash) > 8 {
			shortHash = shortHash[:8]
		}
		commits = append(commits, Commit{
			Hash:        hash,
			ShortHash:   shortHash,
			AuthorName:  item.Author.Name,
			AuthorEmail: item.Author.Email,
			Message:     strings.TrimSpace(item.Message),
			CommittedAt: item.Author.When.UTC().Format("2006-01-02T15:04:05Z"),
		})
		count++
		return nil
	})
	if err != nil && !errors.Is(err, errStopIteration) {
		return nil, fmt.Errorf("iterate commits: %w", err)
	}
	return commits, nil
}

func (s *Service) Search(ctx context.Context, repoPath string, refName string, defaultBranch string, input SearchParams) ([]SearchResult, error) {
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

	limit := input.Limit
	if limit <= 0 {
		limit = 20
	}
	if limit > 500 {
		limit = 500
	}
	maxFiles := input.MaxFiles
	if maxFiles <= 0 {
		maxFiles = 300
	}
	if maxFiles > 5000 {
		maxFiles = 5000
	}

	maxFileSize := input.MaxFileSize
	if maxFileSize <= 0 {
		maxFileSize = 256 * 1024
	}
	if maxFileSize > 10*1024*1024 {
		maxFileSize = 10 * 1024 * 1024
	}

	matcher, err := buildSearchMatcher(query, input.MatchCase, input.UseRegex)
	if err != nil {
		return nil, err
	}

	pathPrefix := normalizePathPrefix(input.Path)
	results := make([]SearchResult, 0, limit)
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
		if file.Blob.Size > maxFileSize {
			return nil
		}
		content, readErr := readBlobContent(file)
		if readErr != nil {
			return nil
		}
		if !isReadableContent(content) {
			return nil
		}

		for lineNumber, line := range strings.Split(string(content), "\n") {
			column, matchLength, matched := matchLine(line, matcher)
			if !matched {
				continue
			}
			results = append(results, SearchResult{
				Path:        file.Name,
				LineNumber:  lineNumber + 1,
				Column:      column,
				MatchLength: matchLength,
				LineContent: line,
			})
			if len(results) >= limit {
				return errStopIteration
			}
		}

		fileCount++
		return nil
	})
	if err != nil && !errors.Is(err, errStopIteration) {
		return nil, fmt.Errorf("search repository: %w", err)
	}
	return results, nil
}

func (s *Service) AnalyzeLanguages(ctx context.Context, repoPath string, refName string, defaultBranch string) (LanguageAnalysis, error) {
	repository, err := s.openRepository(repoPath)
	if err != nil {
		return LanguageAnalysis{}, err
	}
	commit, err := s.resolveCommit(ctx, repository, refName, defaultBranch)
	if err != nil {
		if errors.Is(err, ErrEmptyRepository) {
			return LanguageAnalysis{Languages: []LanguageStat{}}, nil
		}
		return LanguageAnalysis{}, err
	}
	tree, err := commit.Tree()
	if err != nil {
		return LanguageAnalysis{}, fmt.Errorf("load commit tree: %w", err)
	}
	bytesByLanguage := map[string]int64{}
	total := int64(0)
	err = tree.Files().ForEach(func(file *object.File) error {
		if ctx.Err() != nil {
			return ctx.Err()
		}
		language := detectLanguage(file.Name)
		if language == "" {
			return nil
		}
		bytesByLanguage[language] += file.Blob.Size
		total += file.Blob.Size
		return nil
	})
	if err != nil {
		return LanguageAnalysis{}, fmt.Errorf("analyze repository languages: %w", err)
	}
	languages := make([]LanguageStat, 0, len(bytesByLanguage))
	for language, bytes := range bytesByLanguage {
		percentage := float64(0)
		if total > 0 {
			percentage = float64(bytes) * 100 / float64(total)
		}
		languages = append(languages, LanguageStat{Language: language, Bytes: bytes, Percentage: percentage})
	}
	slices.SortFunc(languages, func(a LanguageStat, b LanguageStat) int {
		if a.Bytes != b.Bytes {
			if a.Bytes > b.Bytes {
				return -1
			}
			return 1
		}
		return strings.Compare(a.Language, b.Language)
	})
	return LanguageAnalysis{
		Revision:   commit.Hash.String(),
		TotalBytes: total,
		Languages:  languages,
	}, nil
}

type searchMatcher struct {
	regex *regexp.Regexp
	query string
	raw   string
}

func buildSearchMatcher(query string, matchCase bool, useRegex bool) (searchMatcher, error) {
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
		return searchMatcher{}, fmt.Errorf("%w: %v", ErrInvalidSearchRegexp, err)
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
	match := target
	if matcher.query == matcher.raw {
		match = target
	}
	end := idx + len(match)
	start := utf8.RuneCountInString(line[:idx]) + 1
	length := utf8.RuneCountInString(line[idx:end])
	return start, length, true
}

func isPathInScope(filePath string, prefix string) bool {
	if filePath == prefix {
		return true
	}
	return strings.HasPrefix(filePath, prefix+"/")
}

func readBlobContent(file *object.File) ([]byte, error) {
	reader, err := file.Reader()
	if err != nil {
		return nil, err
	}
	defer reader.Close()
	return io.ReadAll(reader)
}

func normalizePathPrefix(value string) string {
	return strings.Trim(strings.ReplaceAll(value, "\\", "/"), "/")
}

func isReadableContent(data []byte) bool {
	if !utf8.Valid(data) {
		return false
	}
	return !bytes.Contains(data, []byte("\x00"))
}

func (s *Service) openRepository(repoPath string) (*git.Repository, error) {
	absRepo, err := s.resolveRepoPath(repoPath)
	if err != nil {
		return nil, err
	}
	repository, err := git.PlainOpen(absRepo)
	if err != nil {
		if errors.Is(err, git.ErrRepositoryNotExists) || os.IsNotExist(err) {
			return nil, ErrRepositoryNotFound
		}
		return nil, fmt.Errorf("open git repository: %w", err)
	}
	return repository, nil
}

func (s *Service) resolveTree(ctx context.Context, repository *git.Repository, refName string, defaultBranch string, treePath string) (*object.Tree, error) {
	commit, err := s.resolveCommit(ctx, repository, refName, defaultBranch)
	if err != nil {
		return nil, err
	}
	tree, err := commit.Tree()
	if err != nil {
		return nil, fmt.Errorf("load commit tree: %w", err)
	}
	normalizedPath := strings.Trim(strings.ReplaceAll(treePath, "\\", "/"), "/")
	if normalizedPath == "" {
		return tree, nil
	}
	subTree, err := tree.Tree(normalizedPath)
	if err != nil {
		if errors.Is(err, object.ErrDirectoryNotFound) {
			return nil, fmt.Errorf("%w: %s", ErrPathNotFound, normalizedPath)
		}
		return nil, fmt.Errorf("load tree %s: %w", normalizedPath, err)
	}
	return subTree, nil
}

func (s *Service) resolveCommit(ctx context.Context, repository *git.Repository, refName string, defaultBranch string) (*object.Commit, error) {
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	default:
	}

	candidates := resolveRevisionCandidates(refName, defaultBranch)
	var lastErr error
	for _, candidate := range candidates {
		hash, err := repository.ResolveRevision(plumbing.Revision(candidate))
		if err != nil {
			lastErr = err
			continue
		}
		commit, err := repository.CommitObject(*hash)
		if err != nil {
			lastErr = err
			continue
		}
		return commit, nil
	}
	if lastErr != nil && errors.Is(lastErr, plumbing.ErrReferenceNotFound) {
		return nil, ErrReferenceNotFound
	}
	if lastErr != nil && errors.Is(lastErr, plumbing.ErrObjectNotFound) {
		return nil, ErrEmptyRepository
	}
	if lastErr != nil {
		return nil, fmt.Errorf("resolve commit: %w", lastErr)
	}
	return nil, ErrReferenceNotFound
}

func (s *Service) resolveRepoPath(repoPath string) (string, error) {
	if strings.TrimSpace(repoPath) == "" {
		return "", fmt.Errorf("repo path is required")
	}
	root, err := filepath.Abs(s.repoRoot)
	if err != nil {
		return "", fmt.Errorf("resolve repo root: %w", err)
	}
	absRepo, err := filepath.Abs(filepath.Join(root, repoPath))
	if err != nil {
		return "", fmt.Errorf("resolve repo path: %w", err)
	}
	if absRepo != root && !strings.HasPrefix(absRepo, root+string(filepath.Separator)) {
		return "", fmt.Errorf("repo path escapes repo root")
	}
	return absRepo, nil
}

func resolveRevisionCandidates(refName string, defaultBranch string) []string {
	normalizedRef := strings.TrimSpace(refName)
	candidates := setx.NewOrderedSetWithCapacity[string](5)
	add := func(value string) {
		value = strings.TrimSpace(value)
		if value == "" {
			return
		}
		candidates.Add(value)
	}
	if normalizedRef == "" {
		if strings.TrimSpace(defaultBranch) != "" {
			add("refs/heads/" + strings.TrimSpace(defaultBranch))
			add(strings.TrimSpace(defaultBranch))
		}
		add("HEAD")
		return candidates.Values()
	}
	add("refs/heads/" + normalizedRef)
	add("refs/tags/" + normalizedRef)
	add(normalizedRef)
	return candidates.Values()
}

func buildBlob(file *object.File) (Blob, error) {
	reader, err := file.Reader()
	if err != nil {
		return Blob{}, fmt.Errorf("open blob reader: %w", err)
	}
	defer reader.Close()

	data, err := io.ReadAll(reader)
	if err != nil {
		return Blob{}, fmt.Errorf("read blob content: %w", err)
	}

	blob := Blob{
		Name: path.Base(file.Name),
		Path: file.Name,
		Size: file.Blob.Size,
	}
	if utf8.Valid(data) && !strings.ContainsRune(string(data), '\x00') {
		blob.Encoding = "utf-8"
		blob.Content = string(data)
		return blob, nil
	}
	blob.Encoding = "base64"
	blob.Content = base64.StdEncoding.EncodeToString(data)
	return blob, nil
}

func isReadmeName(name string) bool {
	switch strings.ToLower(strings.TrimSpace(name)) {
	case "readme", "readme.md", "readme.txt", "readme.rst", "readme.adoc":
		return true
	default:
		return false
	}
}

func detectLanguage(filePath string) string {
	name := strings.ToLower(path.Base(filePath))
	ext := strings.ToLower(path.Ext(filePath))
	switch name {
	case "dockerfile":
		return "Dockerfile"
	case "makefile":
		return "Makefile"
	case "go.mod", "go.sum":
		return "Go"
	case "cargo.toml", "cargo.lock":
		return "Rust"
	case "package.json", "pnpm-lock.yaml", "package-lock.json", "yarn.lock":
		return "JavaScript"
	}
	switch ext {
	case ".go":
		return "Go"
	case ".rs":
		return "Rust"
	case ".java":
		return "Java"
	case ".kt", ".kts":
		return "Kotlin"
	case ".js", ".mjs", ".cjs":
		return "JavaScript"
	case ".ts":
		return "TypeScript"
	case ".jsx":
		return "JavaScript JSX"
	case ".tsx":
		return "TypeScript JSX"
	case ".css":
		return "CSS"
	case ".scss", ".sass":
		return "SCSS"
	case ".html", ".htm":
		return "HTML"
	case ".md", ".markdown":
		return "Markdown"
	case ".json":
		return "JSON"
	case ".yaml", ".yml":
		return "YAML"
	case ".toml":
		return "TOML"
	case ".xml", ".pom":
		return "XML"
	case ".sql":
		return "SQL"
	case ".sh", ".bash":
		return "Shell"
	case ".ps1":
		return "PowerShell"
	case ".py":
		return "Python"
	case ".rb":
		return "Ruby"
	case ".php":
		return "PHP"
	case ".c", ".h":
		return "C"
	case ".cc", ".cpp", ".cxx", ".hpp", ".hh":
		return "C++"
	default:
		return ""
	}
}
