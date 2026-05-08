package project

import (
	"context"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	gitports "github.com/DaiYuANg/gity/internal/application/ports"
)

func (s *Service) ListTree(ctx context.Context, id int64, refName, treePath string) ([]gitports.TreeEntry, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	entries, err := s.gitRepository.ListTree(ctx, repositoryPath(project), refName, project.DefaultBranch, treePath)
	if err != nil {
		return nil, mapGitError(err)
	}
	return entries, nil
}

func (s *Service) Search(ctx context.Context, id int64, refName, query, path string, limit, maxFiles int, maxFileSize int64, matchCase, useRegex bool) ([]gitports.SearchResult, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	results, err := s.gitRepository.Search(ctx, repositoryPath(project), refName, project.DefaultBranch, gitports.SearchParams{
		Query:       query,
		Path:        path,
		Limit:       limit,
		MaxFiles:    maxFiles,
		MaxFileSize: maxFileSize,
		MatchCase:   matchCase,
		UseRegex:    useRegex,
	})
	if err != nil {
		return nil, mapGitError(err)
	}
	return results, nil
}

func (s *Service) GetBlob(ctx context.Context, id int64, refName, blobPath string) (gitports.Blob, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return gitports.Blob{}, apperror.NotFound("project not found", err)
	}
	blob, err := s.gitRepository.GetBlob(ctx, repositoryPath(project), refName, project.DefaultBranch, blobPath)
	if err != nil {
		return gitports.Blob{}, mapGitError(err)
	}
	return blob, nil
}

func (s *Service) GetReadme(ctx context.Context, id int64, refName string) (gitports.Blob, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return gitports.Blob{}, apperror.NotFound("project not found", err)
	}
	blob, err := s.gitRepository.GetReadme(ctx, repositoryPath(project), refName, project.DefaultBranch)
	if err != nil {
		return gitports.Blob{}, mapGitError(err)
	}
	return blob, nil
}

func (s *Service) ListCommits(ctx context.Context, id int64, refName string, limit int) ([]gitports.Commit, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	commits, err := s.gitRepository.ListCommits(ctx, repositoryPath(project), refName, project.DefaultBranch, limit)
	if err != nil {
		return nil, mapGitError(err)
	}
	return commits, nil
}

func (s *Service) AnalyzeLanguages(ctx context.Context, id int64, refName string) (gitports.LanguageAnalysis, error) {
	project, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return gitports.LanguageAnalysis{}, apperror.NotFound("project not found", err)
	}
	analysis, err := s.gitRepository.AnalyzeLanguages(ctx, repositoryPath(project), refName, project.DefaultBranch)
	if err != nil {
		return gitports.LanguageAnalysis{}, mapGitError(err)
	}
	return analysis, nil
}
