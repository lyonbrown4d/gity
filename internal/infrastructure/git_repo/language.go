package gitrepo

import (
	"context"
	"errors"
	"fmt"
	"strings"

	collectionlist "github.com/arcgolabs/collectionx/list"
	enry "github.com/go-enry/go-enry/v2"
	"github.com/go-git/go-git/v5/plumbing/object"
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
		content, err := readBlobContent(file)
		if err != nil {
			return err
		}
		language := detectLanguage(file.Name, content)
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
	return buildLanguageStats(bytesByLanguage, total), total, nil
}

func buildLanguageStats(bytesByLanguage map[string]int64, total int64) []LanguageStat {
	languages := collectionlist.NewListWithCapacity[LanguageStat](len(bytesByLanguage))
	for language, bytes := range bytesByLanguage {
		languages.Add(LanguageStat{Language: language, Bytes: bytes, Percentage: languagePercentage(bytes, total)})
	}
	sortLanguageStats(languages)
	return languages.Values()
}

func languagePercentage(bytes, total int64) float64 {
	if total <= 0 {
		return 0
	}
	return float64(bytes) * 100 / float64(total)
}

func sortLanguageStats(languages *collectionlist.List[LanguageStat]) {
	languages.Sort(func(a, b LanguageStat) int {
		if a.Bytes != b.Bytes {
			if a.Bytes > b.Bytes {
				return -1
			}
			return 1
		}
		return strings.Compare(a.Language, b.Language)
	})
}

func detectLanguage(filePath string, content []byte) string {
	if enry.IsVendor(filePath) || enry.IsBinary(content) || enry.IsGenerated(filePath, content) {
		return ""
	}
	return enry.GetLanguage(filePath, content)
}
