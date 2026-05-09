package searchindex

import (
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	git "github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing"
	"github.com/go-git/go-git/v5/plumbing/object"
	"github.com/samber/oops"
)

const revisionFileName = ".gity_revision"

func (s *Service) openRepository(project projectdomain.Project) (*git.Repository, error) {
	repoPath, err := s.projectRepoPath(project)
	if err != nil {
		return nil, err
	}
	repository, err := git.PlainOpen(repoPath)
	if err != nil {
		return nil, oops.In("search_index").With("project_id", project.ID, "repo_path", repoPath).Wrapf(err, "open project repository")
	}
	return repository, nil
}

func (s *Service) projectRepoPath(project projectdomain.Project) (string, error) {
	root, err := filepath.Abs(s.repoRoot)
	if err != nil {
		return "", oops.In("search_index").Wrapf(err, "resolve git repo root")
	}
	repoPath, err := filepath.Abs(filepath.Join(root, filepath.FromSlash(project.FullPath+".git")))
	if err != nil {
		return "", oops.In("search_index").With("project_id", project.ID, "full_path", project.FullPath).Wrapf(err, "resolve project repo path")
	}
	if repoPath != root && !strings.HasPrefix(repoPath, root+string(filepath.Separator)) {
		return "", oops.In("search_index").With("project_id", project.ID, "repo_path", repoPath).New("project repo path escapes repo root")
	}
	return repoPath, nil
}

func resolveProjectCommit(repository *git.Repository, defaultBranch string) (*object.Commit, error) {
	branch := strings.TrimSpace(defaultBranch)
	candidates := []string{"HEAD"}
	if branch != "" {
		candidates = []string{"refs/heads/" + branch, branch, "HEAD"}
	}
	var lastErr error
	for _, candidate := range candidates {
		hash, err := repository.ResolveRevision(plumbing.Revision(candidate))
		if err != nil {
			lastErr = err
			continue
		}
		commit, err := repository.CommitObject(*hash)
		if err != nil {
			lastErr = err
			continue
		}
		return commit, nil
	}
	return nil, lastErr
}

func currentRevision(projectIndexPath string) string {
	root, err := os.OpenRoot(projectIndexPath)
	if err != nil {
		return ""
	}
	raw, readErr := root.ReadFile(revisionFileName)
	closeErr := root.Close()
	if readErr != nil || closeErr != nil {
		return ""
	}
	return strings.TrimSpace(string(raw))
}

func (s *Service) projectIndexPath(projectID int64) (string, error) {
	root, err := s.indexRoot()
	if err != nil {
		return "", err
	}
	return filepath.Join(root, strconv.FormatInt(projectID, 10)), nil
}

func (s *Service) indexRoot() (string, error) {
	root := strings.TrimSpace(s.settings.IndexRoot)
	if root == "" {
		root = "./data/search-index"
	}
	absRoot, err := filepath.Abs(root)
	if err != nil {
		return "", oops.In("search_index").With("index_root", root).Wrapf(err, "resolve search index root")
	}
	return absRoot, nil
}

func (s *Service) refreshInterval() time.Duration {
	seconds := s.settings.RefreshIntervalSeconds
	if seconds <= 0 {
		seconds = 300
	}
	return time.Duration(seconds) * time.Second
}

func normalizeMaxFiles(value int) int {
	if value <= 0 {
		return 5000
	}
	if value > 100000 {
		return 100000
	}
	return value
}

func normalizeMaxFileSize(value int64) int64 {
	if value <= 0 {
		return 512 * 1024
	}
	if value > 10*1024*1024 {
		return 10 * 1024 * 1024
	}
	return value
}
