package ports

import (
	"context"

	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
)

type CodeSearchIndex interface {
	SearchProject(ctx context.Context, project projectdomain.Project, refName string, input SearchParams) (CodeSearchIndexResult, error)
	RefreshProject(ctx context.Context, project projectdomain.Project) error
	DeleteProject(ctx context.Context, projectID int64) error
}

type CodeSearchIndexResult struct {
	Results []SearchResult
	Hit     bool
}
