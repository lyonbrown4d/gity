package gitrepo

import (
	"context"
	"encoding/base64"
	"errors"
	"fmt"
	"io"
	"os"
	"path"
	"path/filepath"
	"strings"
	"unicode/utf8"

	setx "github.com/arcgolabs/collectionx/set"
	git "github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git/go-git/v5/plumbing/object"
	"github.com/samber/oops"
)

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

func (s *Service) resolveTree(ctx context.Context, repository *git.Repository, refName, defaultBranch, treePath string) (*object.Tree, error) {
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

func (s *Service) resolveCommit(ctx context.Context, repository *git.Repository, refName, defaultBranch string) (*object.Commit, error) {
	select {
	case <-ctx.Done():
		return nil, oops.In("git_repo").Wrap(ctx.Err())
	default:
	}

	candidates := resolveRevisionCandidates(refName, defaultBranch)
	var lastErr error
	for _, candidate := range candidates {
		commit, err := resolveCandidateCommit(repository, candidate)
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

func resolveCandidateCommit(repository *git.Repository, candidate string) (*object.Commit, error) {
	hash, err := repository.ResolveRevision(plumbing.Revision(candidate))
	if err != nil {
		return nil, fmt.Errorf("resolve revision %s: %w", candidate, err)
	}
	commit, err := repository.CommitObject(*hash)
	if err != nil {
		return nil, fmt.Errorf("load commit %s: %w", candidate, err)
	}
	return commit, nil
}

func (s *Service) resolveRepoPath(repoPath string) (string, error) {
	if strings.TrimSpace(repoPath) == "" {
		return "", errors.New("repo path is required")
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
		return "", errors.New("repo path escapes repo root")
	}
	return absRepo, nil
}

func resolveRevisionCandidates(refName, defaultBranch string) []string {
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

func buildBlob(file *object.File) (blob Blob, err error) {
	reader, err := file.Reader()
	if err != nil {
		return Blob{}, fmt.Errorf("open blob reader: %w", err)
	}
	defer func() {
		if closeErr := reader.Close(); closeErr != nil {
			if err != nil {
				err = oops.In("git_repo").With("path", file.Name).Wrapf(oops.Join(err, closeErr), "build blob and close reader")
				return
			}
			err = oops.In("git_repo").With("path", file.Name).Wrapf(closeErr, "close blob reader")
		}
	}()

	data, err := io.ReadAll(reader)
	if err != nil {
		return Blob{}, fmt.Errorf("read blob content: %w", err)
	}

	blob = Blob{
		Name: path.Base(file.Name),
		Path: file.Name,
		Size: file.Size,
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
