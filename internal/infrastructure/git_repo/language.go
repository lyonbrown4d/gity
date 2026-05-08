package gitrepo

import (
	"context"
	"errors"
	"fmt"
	"path"
	"strings"

	collectionlist "github.com/arcgolabs/collectionx/list"
	mappingx "github.com/arcgolabs/collectionx/mapping"
	"github.com/go-git/go-git/v5/plumbing/object"
)

var (
	fileNameLanguages = mappingx.NewMapFrom(map[string]string{
		"dockerfile":        "Dockerfile",
		"makefile":          "Makefile",
		"go.mod":            "Go",
		"go.sum":            "Go",
		"cargo.toml":        "Rust",
		"cargo.lock":        "Rust",
		"package.json":      "JavaScript",
		"pnpm-lock.yaml":    "JavaScript",
		"package-lock.json": "JavaScript",
		"yarn.lock":         "JavaScript",
	})
	fileExtensionLanguages = mappingx.NewMapFrom(map[string]string{
		".go":       "Go",
		".rs":       "Rust",
		".java":     "Java",
		".kt":       "Kotlin",
		".kts":      "Kotlin",
		".js":       "JavaScript",
		".mjs":      "JavaScript",
		".cjs":      "JavaScript",
		".ts":       "TypeScript",
		".jsx":      "JavaScript JSX",
		".tsx":      "TypeScript JSX",
		".css":      "CSS",
		".scss":     "SCSS",
		".sass":     "SCSS",
		".html":     "HTML",
		".htm":      "HTML",
		".md":       "Markdown",
		".markdown": "Markdown",
		".json":     "JSON",
		".yaml":     "YAML",
		".yml":      "YAML",
		".toml":     "TOML",
		".xml":      "XML",
		".pom":      "XML",
		".sql":      "SQL",
		".sh":       "Shell",
		".bash":     "Shell",
		".ps1":      "PowerShell",
		".py":       "Python",
		".rb":       "Ruby",
		".php":      "PHP",
		".c":        "C",
		".h":        "C",
		".cc":       "C++",
		".cpp":      "C++",
		".cxx":      "C++",
		".hpp":      "C++",
		".hh":       "C++",
	})
)

func (s *Service) AnalyzeLanguages(ctx context.Context, repoPath, refName, defaultBranch string) (LanguageAnalysis, error) {
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
	stats, total, err := analyzeLanguageBytes(ctx, tree)
	if err != nil {
		return LanguageAnalysis{}, err
	}
	return LanguageAnalysis{
		Revision:   commit.Hash.String(),
		TotalBytes: total,
		Languages:  stats,
	}, nil
}

func analyzeLanguageBytes(ctx context.Context, tree *object.Tree) ([]LanguageStat, int64, error) {
	bytesByLanguage := map[string]int64{}
	total := int64(0)
	err := tree.Files().ForEach(func(file *object.File) error {
		if ctx.Err() != nil {
			return ctx.Err()
		}
		language := detectLanguage(file.Name)
		if language == "" {
			return nil
		}
		bytesByLanguage[language] += file.Size
		total += file.Size
		return nil
	})
	if err != nil {
		return nil, 0, fmt.Errorf("analyze repository languages: %w", err)
	}

	languages := collectionlist.NewListWithCapacity[LanguageStat](len(bytesByLanguage))
	for language, bytes := range bytesByLanguage {
		percentage := float64(0)
		if total > 0 {
			percentage = float64(bytes) * 100 / float64(total)
		}
		languages.Add(LanguageStat{Language: language, Bytes: bytes, Percentage: percentage})
	}
	languages.Sort(func(a, b LanguageStat) int {
		if a.Bytes != b.Bytes {
			if a.Bytes > b.Bytes {
				return -1
			}
			return 1
		}
		return strings.Compare(a.Language, b.Language)
	})
	return languages.Values(), total, nil
}

func detectLanguage(filePath string) string {
	name := strings.ToLower(path.Base(filePath))
	if language, ok := fileNameLanguages.Get(name); ok {
		return language
	}
	language, _ := fileExtensionLanguages.Get(strings.ToLower(path.Ext(filePath)))
	return language
}
