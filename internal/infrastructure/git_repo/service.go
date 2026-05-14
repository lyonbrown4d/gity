package gitrepo

import (
	"context"
	"errors"
	"fmt"
	"strings"

	collectionlist "github.com/arcgolabs/collectionx/list"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git/go-git/v5/plumbing/storer"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	"github.com/lyonbrown4d/gity/internal/config"
	"github.com/samber/oops"
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
		return handleBranchIteratorError(err)
	}
	defer iter.Close()

	branches, err := collectBranches(ctx, iter, strings.TrimSpace(defaultBranch))
	if err != nil {
		return nil, fmt.Errorf("iterate branches: %w", err)
	}
	sortBranches(branches)
	return branches.Values(), nil
}

func handleBranchIteratorError(err error) ([]Branch, error) {
	if errors.Is(err, plumbing.ErrReferenceNotFound) {
		return []Branch{}, nil
	}
	return nil, fmt.Errorf("list branches: %w", err)
}

func collectBranches(ctx context.Context, iter storer.ReferenceIter, defaultBranch string) (*collectionlist.List[Branch], error) {
	branches := collectionlist.NewList[Branch]()
	err := iter.ForEach(func(ref *plumbing.Reference) error {
		return collectBranch(ctx, branches, ref, defaultBranch)
	})
	if err != nil {
		return nil, oops.In("git_repo").Wrapf(err, "collect branches")
	}
	return branches, nil
}

func collectBranch(ctx context.Context, branches *collectionlist.List[Branch], ref *plumbing.Reference, defaultBranch string) error {
	if ctx.Err() != nil {
		return oops.In("git_repo").With("branch", ref.Name().Short()).Wrapf(ctx.Err(), "collect branch canceled")
	}
	branchName := ref.Name().Short()
	branches.Add(Branch{
		Name:      branchName,
		Hash:      ref.Hash().String(),
		IsDefault: branchName == defaultBranch,
	})
	return nil
}

func sortBranches(branches *collectionlist.List[Branch]) {
	branches.Sort(func(a, b Branch) int {
		if a.IsDefault && !b.IsDefault {
			return -1
		}
		if !a.IsDefault && b.IsDefault {
			return 1
		}
		return strings.Compare(a.Name, b.Name)
	})
}
