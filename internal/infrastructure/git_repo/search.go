package gitrepo

import (
	"context"
	"errors"
	"fmt"
	"io"

	collectionlist "github.com/arcgolabs/collectionx/list"
	"github.com/go-git/go-git/v5/plumbing/object"
	gitsearch "github.com/lyonbrown4d/gity/internal/infrastructure/git_search"
	"github.com/samber/oops"
)

func (s *Service) Search(ctx context.Context, repoPath, refName, defaultBranch string, input SearchParams) ([]SearchResult, error) {
	plan, err := gitsearch.NewPlan(input)
	if err != nil {
		return nil, oops.In("git_repo").Wrapf(err, "build repository search plan")
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

func (s *Service) searchTree(ctx context.Context, repoPath string, tree *object.Tree, plan gitsearch.Plan) ([]SearchResult, error) {
	results := collectionlist.NewListWithCapacity[SearchResult](plan.Limit())
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

func (s *Service) searchTreeFile(ctx context.Context, repoPath string, file *object.File, plan gitsearch.Plan, state *searchTreeState, results *collectionlist.List[SearchResult]) error {
	if ctx.Err() != nil {
		return oops.In("git_repo").With("repo_path", repoPath).Wrapf(ctx.Err(), "search repository canceled")
	}
	if state.fileCount >= plan.MaxFiles() {
		return errStopIteration
	}
	searched, err := s.searchFile(repoPath, file, plan, results)
	if err != nil {
		return err
	}
	if searched {
		state.fileCount++
	}
	if results.Len() >= plan.Limit() {
		return errStopIteration
	}
	return nil
}

func (s *Service) searchFile(repoPath string, file *object.File, plan gitsearch.Plan, results *collectionlist.List[SearchResult]) (bool, error) {
	if plan.PathPrefix() != "" && !gitsearch.IsPathInScope(file.Name, plan.PathPrefix()) {
		return false, nil
	}
	if file.Size > plan.MaxFileSize() {
		return false, nil
	}
	content, readErr := readBlobContent(file)
	if readErr != nil {
		return false, oops.In("git_repo").With("repo_path", repoPath, "path", file.Name).Wrapf(readErr, "read searchable blob")
	}
	if !gitsearch.IsReadableContent(content) {
		return true, nil
	}
	gitsearch.AppendMatches(file.Name, content, plan, results)
	return true, nil
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
