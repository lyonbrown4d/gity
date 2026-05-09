package runneragent

import (
	"encoding/json"
	"io/fs"
	"os"
	"path"
	"strings"

	storageports "github.com/DaiYuANg/gity/internal/application/ports"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	collectionlist "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	"github.com/bmatcuk/doublestar/v4"
	"github.com/samber/oops"
)

type ArtifactFile struct {
	Name        string
	FileName    string
	FilePath    string
	ContentType string
	Content     []byte
}

func CollectArtifacts(job cidomain.ProjectJob, result string) (artifacts []ArtifactFile, err error) {
	payload, scriptResult, err := decodeArtifactInputs(job, result)
	if err != nil {
		return nil, err
	}
	if len(payload.Artifacts) == 0 {
		return nil, nil
	}
	workDir := strings.TrimSpace(scriptResult.WorkDir)
	if workDir == "" {
		return nil, oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID).New("script result does not include work_dir")
	}
	return collectArtifactsFromWorkDir(job, payload, workDir)
}

func decodeArtifactInputs(job cidomain.ProjectJob, result string) (ScriptPayload, ScriptResult, error) {
	var payload ScriptPayload
	if decodeErr := json.Unmarshal([]byte(strings.TrimSpace(job.Payload)), &payload); decodeErr != nil {
		return ScriptPayload{}, ScriptResult{}, oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID).Wrapf(decodeErr, "decode script job payload")
	}
	var scriptResult ScriptResult
	if decodeErr := json.Unmarshal([]byte(strings.TrimSpace(result)), &scriptResult); decodeErr != nil {
		return ScriptPayload{}, ScriptResult{}, oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID).Wrapf(decodeErr, "decode script result")
	}
	return payload, scriptResult, nil
}

func collectArtifactsFromWorkDir(job cidomain.ProjectJob, payload ScriptPayload, workDir string) (artifacts []ArtifactFile, err error) {
	rootHandle, err := os.OpenRoot(workDir)
	if err != nil {
		return nil, oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID, "work_dir", workDir).Wrapf(err, "open artifact work dir")
	}
	defer func() {
		if closeErr := rootHandle.Close(); closeErr != nil && err == nil {
			err = oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID, "work_dir", workDir).Wrapf(closeErr, "close artifact work dir")
		}
	}()
	files := collectionlist.NewList[ArtifactFile]()
	seen := setx.NewSet[string]()
	for _, pattern := range payload.Artifacts {
		if err := collectArtifactPattern(job, rootHandle, pattern, seen, files); err != nil {
			return nil, err
		}
	}
	return files.Values(), nil
}

func collectArtifactPattern(job cidomain.ProjectJob, rootHandle *os.Root, pattern string, seen *setx.Set[string], files *collectionlist.List[ArtifactFile]) error {
	root := rootHandle.FS()
	matches, err := doublestar.Glob(root, normalizeArtifactPattern(pattern), doublestar.WithFilesOnly())
	if err != nil {
		return oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID, "pattern", pattern).Wrapf(err, "match artifact pattern")
	}
	for _, match := range matches {
		if err := addArtifactMatch(job, rootHandle, match, seen, files); err != nil {
			return err
		}
	}
	return nil
}

func addArtifactMatch(job cidomain.ProjectJob, rootHandle *os.Root, match string, seen *setx.Set[string], files *collectionlist.List[ArtifactFile]) error {
	relative := normalizeArtifactPath(match)
	if relative == "" || seen.Contains(relative) {
		return nil
	}
	seen.Add(relative)
	root := rootHandle.FS()
	info, err := fs.Stat(root, relative)
	if err != nil {
		return oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID, "artifact_path", relative).Wrapf(err, "stat artifact")
	}
	if info.IsDir() {
		return nil
	}
	content, err := rootHandle.ReadFile(relative)
	if err != nil {
		return oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID, "artifact_path", relative).Wrapf(err, "read artifact")
	}
	files.Add(ArtifactFile{Name: relative, FileName: path.Base(relative), FilePath: relative, ContentType: detectArtifactContentType(relative, content), Content: content})
	return nil
}

func normalizeArtifactPattern(value string) string {
	trimmed := strings.Trim(strings.ReplaceAll(strings.TrimSpace(value), "\\", "/"), "/")
	if trimmed == "" {
		return "noop-never-match"
	}
	return trimmed
}

func normalizeArtifactPath(value string) string {
	trimmed := path.Clean(strings.Trim(strings.ReplaceAll(strings.TrimSpace(value), "\\", "/"), "/"))
	if trimmed == "." || strings.HasPrefix(trimmed, "../") || trimmed == ".." {
		return ""
	}
	return trimmed
}

func detectArtifactContentType(fileName string, content []byte) string {
	return storageports.DetectContentType(fileName, content)
}
