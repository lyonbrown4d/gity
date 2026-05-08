package gitrepo

import (
	"context"
	"errors"
	"fmt"
	"strings"

	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	"github.com/DaiYuANg/gity/internal/config"
	collectionlist "github.com/arcgolabs/collectionx/list"
	"github.com/go-git/go-git/v5/plumbing"
)

var (
	ErrRepositoryNotFound  = gitports.ErrRepositoryNotFound
	ErrReferenceNotFound   = gitports.ErrReferenceNotFound
	ErrPathNotFound        = gitports.ErrPathNotFound
	ErrReadmeNotFound      = gitports.ErrReadmeNotFound
	ErrEmptyRepository     = gitports.ErrEmptyRepository
	ErrInvalidSearchQuery  = gitports.ErrInvalidSearchQuery
	ErrInvalidSearchRegexp = gitports.ErrInvalidSearchRegexp
	errStopIteration       = errors.New("stop iteration")
)

type Service struct {
	repoRoot string
}

type Branch = gitports.Branch
type TreeEntry = gitports.TreeEntry
type Blob = gitports.Blob
type Commit = gitports.Commit
type SearchResult = gitports.SearchResult
type LanguageStat = gitports.LanguageStat
type LanguageAnalysis = gitports.LanguageAnalysis
type SearchParams = gitports.SearchParams

func NewService(settings config.Settings) *Service {
	return &Service{repoRoot: settings.Git.RepoRoot}
}

func (s *Service) ListBranches(ctx context.Context, repoPath, defaultBranch string) ([]Branch, error) {
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

	branches := collectionlist.NewList[Branch]()
	err = iter.ForEach(func(ref *plumbing.Reference) error {
		if ctx.Err() != nil {
			return ctx.Err()
		}
		branches.Add(Branch{
			Name:      ref.Name().Short(),
			Hash:      ref.Hash().String(),
			IsDefault: ref.Name().Short() == strings.TrimSpace(defaultBranch),
		})
		return nil
	})
	if err != nil {
		return nil, fmt.Errorf("iterate branches: %w", err)
	}
	branches.Sort(func(a, b Branch) int {
		if a.IsDefault && !b.IsDefault {
			return -1
		}
		if !a.IsDefault && b.IsDefault {
			return 1
		}
		return strings.Compare(a.Name, b.Name)
	})
	return branches.Values(), nil
}
