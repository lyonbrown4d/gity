package searchindex

import (
	"context"
	"errors"
	"os"
	"strings"

	"github.com/arcgolabs/collectionx/list"
	"github.com/blevesearch/bleve/v2"
	"github.com/blevesearch/bleve/v2/search"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	gitsearch "github.com/lyonbrown4d/gity/internal/infrastructure/git_search"
	"github.com/samber/oops"
)

func NewCodeSearchIndex(service *Service) gitports.CodeSearchIndex {
	return service
}

func (s *Service) SearchProject(ctx context.Context, project projectdomain.Project, refName string, input gitports.SearchParams) (gitports.CodeSearchIndexResult, error) {
	if !s.canQueryProjectIndex(refName, project.DefaultBranch, input) {
		return gitports.CodeSearchIndexResult{}, nil
	}
	plan, err := gitsearch.NewPlan(input)
	if err != nil {
		return gitports.CodeSearchIndexResult{}, oops.In("search_index").With("project_id", project.ID).Wrapf(err, "build project search index query plan")
	}
	indexed, indexErr := s.projectIndexMatches(ctx, project)
	if indexErr != nil || !indexed {
		return gitports.CodeSearchIndexResult{}, indexErr
	}
	return s.queryProjectIndex(ctx, project.ID, plan)
}

func (s *Service) canQueryProjectIndex(refName, defaultBranch string, input gitports.SearchParams) bool {
	return s.settings.IndexEnabled && !input.UseRegex && isDefaultBranchRef(refName, defaultBranch)
}

func (s *Service) queryProjectIndex(ctx context.Context, projectID int64, plan gitsearch.Plan) (gitports.CodeSearchIndexResult, error) {
	index, err := s.openProjectIndex(projectID)
	if errors.Is(err, os.ErrNotExist) {
		return gitports.CodeSearchIndexResult{}, nil
	}
	if err != nil {
		return gitports.CodeSearchIndexResult{}, err
	}
	defer func() {
		if closeErr := index.Close(); closeErr != nil {
			s.logError("close project search query index failed", closeErr, slogProjectID(projectID))
		}
	}()

	results, err := searchProjectIndex(ctx, index, plan)
	if err != nil {
		return gitports.CodeSearchIndexResult{}, oops.In("search_index").With("project_id", projectID).Wrapf(err, "query project search index")
	}
	return gitports.CodeSearchIndexResult{Results: results, Hit: true}, nil
}

func (s *Service) DeleteProject(ctx context.Context, projectID int64) error {
	if err := ctx.Err(); err != nil {
		return oops.In("search_index").With("project_id", projectID).Wrapf(err, "delete project search index canceled")
	}
	indexPath, err := s.projectIndexPath(projectID)
	if err != nil {
		return err
	}
	if err := os.RemoveAll(indexPath); err != nil {
		return oops.In("search_index").With("project_id", projectID, "index_path", indexPath).Wrapf(err, "delete project search index")
	}
	return nil
}

func isDefaultBranchRef(refName, defaultBranch string) bool {
	ref := strings.TrimSpace(refName)
	branch := strings.TrimSpace(defaultBranch)
	if ref == "" || ref == branch || ref == "refs/heads/"+branch {
		return true
	}
	return ref == "HEAD"
}

func (s *Service) projectIndexMatches(ctx context.Context, project projectdomain.Project) (bool, error) {
	repository, err := s.openRepository(project)
	if shouldIgnoreRepositoryError(err) {
		return false, nil
	}
	if err != nil {
		return false, err
	}
	commit, err := resolveProjectCommit(repository, project.DefaultBranch)
	if shouldIgnoreCommitError(err) {
		return false, nil
	}
	if err != nil {
		return false, oops.In("search_index").With("project_id", project.ID, "default_branch", project.DefaultBranch).Wrapf(err, "resolve project default branch for search index query")
	}
	indexPath, err := s.projectIndexPath(project.ID)
	if err != nil {
		return false, err
	}
	if err := ctx.Err(); err != nil {
		return false, oops.In("search_index").With("project_id", project.ID).Wrapf(err, "check project search index revision canceled")
	}
	return currentRevision(indexPath) == commit.Hash.String(), nil
}

func (s *Service) openProjectIndex(projectID int64) (bleve.Index, error) {
	indexPath, err := s.projectIndexPath(projectID)
	if err != nil {
		return nil, err
	}
	if _, statErr := os.Stat(indexPath); statErr != nil {
		if os.IsNotExist(statErr) {
			return nil, os.ErrNotExist
		}
		return nil, oops.In("search_index").With("project_id", projectID, "index_path", indexPath).Wrapf(statErr, "stat project search index")
	}
	index, err := bleve.Open(indexPath)
	if err != nil {
		return nil, oops.In("search_index").With("project_id", projectID, "index_path", indexPath).Wrapf(err, "open project search index")
	}
	return index, nil
}

func searchProjectIndex(ctx context.Context, index bleve.Index, plan gitsearch.Plan) ([]gitports.SearchResult, error) {
	results := list.NewListWithCapacity[gitports.SearchResult](plan.Limit())
	response, err := index.SearchInContext(ctx, newSearchRequest(plan))
	if err != nil {
		return nil, oops.In("search_index").Wrapf(err, "execute project search index query")
	}
	for _, hit := range response.Hits {
		appendIndexedHitMatches(hit, plan, results)
		if results.Len() >= plan.Limit() {
			break
		}
	}
	return results.Values(), nil
}

func newSearchRequest(plan gitsearch.Plan) *bleve.SearchRequest {
	query := bleve.NewMatchQuery(plan.Query())
	query.SetField("content")
	request := bleve.NewSearchRequestOptions(query, plan.MaxFiles(), 0, false)
	request.Fields = []string{"path", "content"}
	return request
}

func appendIndexedHitMatches(hit *search.DocumentMatch, plan gitsearch.Plan, results *list.List[gitports.SearchResult]) {
	path := fieldString(hit.Fields["path"])
	if path == "" {
		path = hit.ID
	}
	if plan.PathPrefix() != "" && !gitsearch.IsPathInScope(path, plan.PathPrefix()) {
		return
	}
	content := fieldString(hit.Fields["content"])
	if content == "" {
		return
	}
	gitsearch.AppendMatches(path, []byte(content), plan, results)
}

func fieldString(value any) string {
	if text, ok := value.(string); ok {
		return text
	}
	return ""
}
