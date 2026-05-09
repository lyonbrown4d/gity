package searchindex

import (
	"bytes"
	"context"
	"errors"
	"io"
	"os"
	"path/filepath"
	"unicode/utf8"

	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	"github.com/blevesearch/bleve/v2"
	enry "github.com/go-enry/go-enry/v2"
	"github.com/go-git/go-git/v5/plumbing/object"
	"github.com/samber/oops"
)

type document struct {
	ProjectID int64  `json:"project_id"`
	FullPath  string `json:"full_path"`
	Revision  string `json:"revision"`
	Path      string `json:"path"`
	Content   string `json:"content"`
	Language  string `json:"language"`
	Size      int64  `json:"size"`
}

type indexState struct {
	maxFiles     int
	maxFileSize  int64
	visitedFiles int
	indexedFiles int
}

var errStopIndexing = errors.New("stop search indexing")

func (s *Service) rebuildProjectIndex(ctx context.Context, project projectdomain.Project, tree *object.Tree, revision, projectIndexPath string) error {
	tempPath := projectIndexPath + ".tmp"
	if err := prepareTemporaryIndex(project.ID, projectIndexPath, tempPath); err != nil {
		return err
	}
	index, err := createProjectIndex(project.ID, tempPath)
	if err != nil {
		return err
	}
	closeIndex := true
	defer func() {
		if closeIndex {
			s.closeIndexWithLog(index, project.ID, tempPath)
		}
	}()

	state := newIndexState(s.settings.MaxFilesPerProject, s.settings.MaxFileSizeBytes)
	if err := s.indexProjectFiles(ctx, index, project, tree, revision, &state); err != nil {
		return err
	}
	closeIndex = false
	if err := closeProjectIndex(index, project.ID, tempPath); err != nil {
		return err
	}
	if err := writeRevisionFile(project.ID, tempPath, revision); err != nil {
		return err
	}
	if err := promoteProjectIndex(project.ID, projectIndexPath, tempPath); err != nil {
		return err
	}
	s.logInfo("project search index refreshed", slogProjectID(project.ID), slogRevision(revision), slogIndexedFiles(state.indexedFiles))
	return nil
}

func prepareTemporaryIndex(projectID int64, projectIndexPath, tempPath string) error {
	if err := os.RemoveAll(tempPath); err != nil {
		return oops.In("search_index").With("project_id", projectID, "index_path", tempPath).Wrapf(err, "clear temporary search index")
	}
	if err := os.MkdirAll(filepath.Dir(projectIndexPath), 0o750); err != nil {
		return oops.In("search_index").With("project_id", projectID, "index_path", projectIndexPath).Wrapf(err, "create project search index parent")
	}
	return nil
}

func createProjectIndex(projectID int64, tempPath string) (bleve.Index, error) {
	index, err := bleve.New(tempPath, bleve.NewIndexMapping())
	if err != nil {
		return nil, oops.In("search_index").With("project_id", projectID, "index_path", tempPath).Wrapf(err, "create project search index")
	}
	return index, nil
}

func (s *Service) closeIndexWithLog(index bleve.Index, projectID int64, indexPath string) {
	if closeErr := index.Close(); closeErr != nil {
		s.logError("close project search index failed", closeErr, slogProjectID(projectID), slogIndexPath(indexPath))
	}
}

func closeProjectIndex(index bleve.Index, projectID int64, indexPath string) error {
	if err := index.Close(); err != nil {
		return oops.In("search_index").With("project_id", projectID, "index_path", indexPath).Wrapf(err, "close project search index")
	}
	return nil
}

func writeRevisionFile(projectID int64, tempPath, revision string) error {
	if err := os.WriteFile(filepath.Join(tempPath, revisionFileName), []byte(revision), 0o600); err != nil {
		return oops.In("search_index").With("project_id", projectID, "index_path", tempPath).Wrapf(err, "write project search index revision")
	}
	return nil
}

func promoteProjectIndex(projectID int64, projectIndexPath, tempPath string) error {
	if err := os.RemoveAll(projectIndexPath); err != nil {
		return oops.In("search_index").With("project_id", projectID, "index_path", projectIndexPath).Wrapf(err, "remove old project search index")
	}
	if err := os.Rename(tempPath, projectIndexPath); err != nil {
		return oops.In("search_index").With("project_id", projectID, "index_path", projectIndexPath).Wrapf(err, "promote project search index")
	}
	return nil
}

func newIndexState(maxFiles int, maxFileSize int64) indexState {
	return indexState{
		maxFiles:    normalizeMaxFiles(maxFiles),
		maxFileSize: normalizeMaxFileSize(maxFileSize),
	}
}

func (s *Service) indexProjectFiles(
	ctx context.Context,
	index bleve.Index,
	project projectdomain.Project,
	tree *object.Tree,
	revision string,
	state *indexState,
) error {
	err := tree.Files().ForEach(func(file *object.File) error {
		return s.indexFile(ctx, index, project, revision, file, state)
	})
	if err == nil || errors.Is(err, errStopIndexing) {
		return nil
	}
	return oops.In("search_index").With("project_id", project.ID, "revision", revision).Wrapf(err, "iterate project files for search index")
}

func (s *Service) indexFile(ctx context.Context, index bleve.Index, project projectdomain.Project, revision string, file *object.File, state *indexState) error {
	if err := ctx.Err(); err != nil {
		return oops.In("search_index").With("project_id", project.ID, "path", file.Name).Wrapf(err, "index project file canceled")
	}
	if state.visitedFiles >= state.maxFiles {
		return errStopIndexing
	}
	state.visitedFiles++
	if file.Size <= 0 || file.Size > state.maxFileSize || enry.IsVendor(file.Name) {
		return nil
	}
	content, err := readFileContent(file)
	if err != nil {
		return err
	}
	if !isSearchableContent(content) || enry.IsGenerated(file.Name, content) {
		return nil
	}
	if err := index.Index(file.Name, newDocument(project, revision, file, content)); err != nil {
		return oops.In("search_index").With("project_id", project.ID, "path", file.Name).Wrapf(err, "index project file")
	}
	state.indexedFiles++
	return nil
}

func newDocument(project projectdomain.Project, revision string, file *object.File, content []byte) document {
	return document{
		ProjectID: project.ID,
		FullPath:  project.FullPath,
		Revision:  revision,
		Path:      file.Name,
		Content:   string(content),
		Language:  enry.GetLanguage(file.Name, content),
		Size:      file.Size,
	}
}

func readFileContent(file *object.File) (content []byte, err error) {
	reader, err := file.Reader()
	if err != nil {
		return nil, oops.In("search_index").With("path", file.Name).Wrapf(err, "open file content")
	}
	defer func() {
		if closeErr := reader.Close(); closeErr != nil {
			if err != nil {
				err = oops.In("search_index").With("path", file.Name).Wrapf(oops.Join(err, closeErr), "read file content and close reader")
				return
			}
			err = oops.In("search_index").With("path", file.Name).Wrapf(closeErr, "close file content")
		}
	}()
	content, err = io.ReadAll(reader)
	if err != nil {
		return nil, oops.In("search_index").With("path", file.Name).Wrapf(err, "read file content")
	}
	return content, nil
}

func isSearchableContent(content []byte) bool {
	return utf8.Valid(content) && !bytes.Contains(content, []byte("\x00")) && !enry.IsBinary(content)
}
