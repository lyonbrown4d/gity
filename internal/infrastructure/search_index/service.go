package searchindex

import (
	"context"
	"errors"
	"log/slog"
	"os"
	"sync"
	"time"

	gitports "github.com/DaiYuANg/gity/internal/application/ports"
	"github.com/DaiYuANg/gity/internal/config"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	git "github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git/go-git/v5/plumbing/object"
	"github.com/samber/oops"
)

type Service struct {
	logger      *slog.Logger
	settings    config.SearchSettings
	repoRoot    string
	projectRepo gitports.ProjectRepository
	cancel      context.CancelFunc
	done        chan struct{}
	mu          sync.Mutex
}

func NewService(logger *slog.Logger, settings config.Settings, projectRepo gitports.ProjectRepository) *Service {
	return &Service{
		logger:      logger,
		settings:    settings.Search,
		repoRoot:    settings.Git.RepoRoot,
		projectRepo: projectRepo,
	}
}

func (s *Service) Start(ctx context.Context) error {
	if !s.settings.IndexEnabled {
		s.logInfo("search index refresher disabled")
		return nil
	}
	if s.projectRepo == nil {
		return oops.In("search_index").New("project repository is required")
	}
	root, err := s.indexRoot()
	if err != nil {
		return err
	}
	if err := os.MkdirAll(root, 0o750); err != nil {
		return oops.In("search_index").With("index_root", root).Wrapf(err, "create search index root")
	}
	runCtx, cancel := context.WithCancel(context.WithoutCancel(ctx))
	s.cancel = cancel
	s.done = make(chan struct{})
	go s.loop(runCtx)
	s.logInfo("search index refresher started", slog.String("index_root", root), slog.Duration("refresh_interval", s.refreshInterval()))
	return nil
}

func (s *Service) Stop(ctx context.Context) error {
	if s.cancel == nil || s.done == nil {
		return nil
	}
	s.cancel()
	select {
	case <-s.done:
		s.logInfo("search index refresher stopped")
		return nil
	case <-ctx.Done():
		return oops.In("search_index").Wrapf(ctx.Err(), "stop search index refresher")
	}
}

func (s *Service) RefreshAll(ctx context.Context) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	projects, err := s.projectRepo.List(ctx, nil)
	if err != nil {
		return oops.In("search_index").Wrapf(err, "list projects for search indexing")
	}
	projectValues := projects.Values()
	for i := range projectValues {
		project := projectValues[i]
		if err := s.RefreshProject(ctx, project); err != nil {
			s.logError("refresh project search index failed", err, slog.Int64("project_id", project.ID), slog.String("full_path", project.FullPath))
		}
	}
	return nil
}

func (s *Service) RefreshProject(ctx context.Context, project projectdomain.Project) error {
	if err := ensureRefreshContext(ctx, project.ID); err != nil {
		return err
	}
	repository, err := s.openRepository(project)
	if shouldIgnoreRepositoryError(err) {
		return nil
	}
	if err != nil {
		return err
	}
	commit, err := resolveProjectCommit(repository, project.DefaultBranch)
	if shouldIgnoreCommitError(err) {
		return nil
	}
	if err != nil {
		return oops.In("search_index").With("project_id", project.ID, "default_branch", project.DefaultBranch).Wrapf(err, "resolve project default branch")
	}
	return s.refreshProjectCommit(ctx, project, commit)
}

func (s *Service) loop(ctx context.Context) {
	defer close(s.done)
	s.refreshSafely(ctx)
	ticker := time.NewTicker(s.refreshInterval())
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			s.refreshSafely(ctx)
		}
	}
}

func (s *Service) refreshSafely(ctx context.Context) {
	if err := s.RefreshAll(ctx); err != nil {
		s.logError("refresh search index failed", err)
	}
}

func (s *Service) refreshProjectCommit(ctx context.Context, project projectdomain.Project, commit *object.Commit) error {
	revision := commit.Hash.String()
	projectIndexPath, err := s.projectIndexPath(project.ID)
	if err != nil {
		return err
	}
	if currentRevision(projectIndexPath) == revision {
		return nil
	}
	tree, err := commit.Tree()
	if err != nil {
		return oops.In("search_index").With("project_id", project.ID, "revision", revision).Wrapf(err, "load project tree")
	}
	return s.rebuildProjectIndex(ctx, project, tree, revision, projectIndexPath)
}

func ensureRefreshContext(ctx context.Context, projectID int64) error {
	if err := ctx.Err(); err != nil {
		return oops.In("search_index").With("project_id", projectID).Wrapf(err, "refresh project search index canceled")
	}
	return nil
}

func shouldIgnoreRepositoryError(err error) bool {
	return errors.Is(err, git.ErrRepositoryNotExists) || os.IsNotExist(err)
}

func shouldIgnoreCommitError(err error) bool {
	return errors.Is(err, plumbing.ErrReferenceNotFound) || errors.Is(err, plumbing.ErrObjectNotFound)
}
