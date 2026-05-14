package gitrepo

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	collectionlist "github.com/arcgolabs/collectionx/list"
	gogit "github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git/go-git/v5/plumbing/object"
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
type Tag = gitports.Tag
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

func (s *Service) ListTags(ctx context.Context, repoPath string) ([]Tag, error) {
	repository, err := s.openRepository(repoPath)
	if err != nil {
		return nil, err
	}
	iter, err := repository.Tags()
	if err != nil {
		if errors.Is(err, plumbing.ErrReferenceNotFound) {
			return []Tag{}, nil
		}
		return nil, fmt.Errorf("list tags: %w", err)
	}
	defer iter.Close()

	tags := collectionlist.NewList[Tag]()
	if err := iter.ForEach(func(ref *plumbing.Reference) error {
		if ctx.Err() != nil {
			return oops.In("git_repo").With("tag", ref.Name().Short()).Wrapf(ctx.Err(), "collect tag canceled")
		}
		tags.Add(s.collectTag(repository, ref))
		return nil
	}); err != nil {
		return nil, oops.In("git_repo").Wrapf(err, "collect tags")
	}
	sortTags(tags)
	return tags.Values(), nil
}

func (s *Service) collectTag(repository *gogit.Repository, ref *plumbing.Reference) Tag {
	tag := Tag{
		Name:       ref.Name().Short(),
		TargetSHA:  ref.Hash().String(),
		ObjectSHA:  ref.Hash().String(),
		ObjectType: "commit",
	}
	if tagObject, err := repository.TagObject(ref.Hash()); err == nil {
		return annotatedTagView(ref, tagObject)
	}
	if commit, err := repository.CommitObject(ref.Hash()); err == nil {
		tag.CreatedAt = formatGitTime(commit.Committer.When)
	}
	return tag
}

func annotatedTagView(ref *plumbing.Reference, tagObject *object.Tag) Tag {
	return Tag{
		Name:       ref.Name().Short(),
		TargetSHA:  tagObject.Target.String(),
		Message:    strings.TrimSpace(tagObject.Message),
		CreatedAt:  formatGitTime(tagObject.Tagger.When),
		Annotated:  true,
		ObjectSHA:  ref.Hash().String(),
		ObjectType: tagObject.TargetType.String(),
	}
}

func sortTags(tags *collectionlist.List[Tag]) {
	tags.Sort(func(a, b Tag) int {
		if a.CreatedAt != "" && b.CreatedAt != "" && a.CreatedAt != b.CreatedAt {
			return strings.Compare(b.CreatedAt, a.CreatedAt)
		}
		return strings.Compare(a.Name, b.Name)
	})
}

func formatGitTime(value time.Time) string {
	if value.IsZero() {
		return ""
	}
	return value.UTC().Format("2006-01-02T15:04:05Z")
}
