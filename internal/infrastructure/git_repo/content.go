package gitrepo

import (
	"context"
	"errors"
	"fmt"
	"path"
	"strings"

	collectionlist "github.com/arcgolabs/collectionx/list"
	git "github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing/filemode"
	"github.com/go-git/go-git/v5/plumbing/object"
)

func (s *Service) ListTree(ctx context.Context, repoPath, refName, defaultBranch, treePath string) ([]TreeEntry, error) {
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

	entries := collectionlist.NewListWithCapacity[TreeEntry](len(tree.Entries))
	for _, entry := range tree.Entries {
		size := int64(0)
		entryType := "blob"
		if entry.Mode == filemode.Dir {
			entryType = "tree"
		} else {
			file, fileErr := tree.File(entry.Name)
			if fileErr == nil {
				size = file.Size
			}
		}
		entryPath := entry.Name
		if strings.TrimSpace(treePath) != "" {
			entryPath = path.Join(strings.Trim(strings.ReplaceAll(treePath, "\\", "/"), "/"), entry.Name)
		}
		entries.Add(TreeEntry{
			Name: entry.Name,
			Path: entryPath,
			Type: entryType,
			Mode: entry.Mode.String(),
			Size: size,
		})
	}
	entries.Sort(func(a, b TreeEntry) int {
		if a.Type != b.Type {
			if a.Type == "tree" {
				return -1
			}
			return 1
		}
		return strings.Compare(a.Name, b.Name)
	})
	return entries.Values(), nil
}

func (s *Service) GetBlob(ctx context.Context, repoPath, refName, defaultBranch, blobPath string) (Blob, error) {
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

func (s *Service) GetReadme(ctx context.Context, repoPath, refName, defaultBranch string) (Blob, error) {
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

func (s *Service) ListCommits(ctx context.Context, repoPath, refName, defaultBranch string, limit int) ([]Commit, error) {
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

	commits := collectionlist.NewListWithCapacity[Commit](limit)
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
		commits.Add(Commit{
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
	return commits.Values(), nil
}
