package runneragent

import (
	"encoding/json"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	collectionlist "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	"github.com/bmatcuk/doublestar/v4"
	"github.com/samber/oops"
	"io/fs"
	"net/http"
	"os"
	"path"
	"path/filepath"
	"strings"
)

type ArtifactFile struct {
	Name        string
	FileName    string
	FilePath    string
	ContentType string
	Content     []byte
}

func CollectArtifacts(job cidomain.ProjectJob, result string) ([]ArtifactFile, error) {
	var payload ScriptPayload
	if err := json.Unmarshal([]byte(strings.TrimSpace(job.Payload)), &payload); err != nil {
		return nil, oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID).Wrapf(err, "decode script job payload")
	}
	if len(payload.Artifacts) == 0 {
		return nil, nil
	}
	var scriptResult ScriptResult
	if err := json.Unmarshal([]byte(strings.TrimSpace(result)), &scriptResult); err != nil {
		return nil, oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID).Wrapf(err, "decode script result")
	}
	workDir := strings.TrimSpace(scriptResult.WorkDir)
	if workDir == "" {
		return nil, oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID).New("script result does not include work_dir")
	}
	root := os.DirFS(workDir)
	files := collectionlist.NewList[ArtifactFile]()
	seen := setx.NewSet[string]()
	for _, pattern := range payload.Artifacts {
		matches, err := doublestar.Glob(root, normalizeArtifactPattern(pattern), doublestar.WithFilesOnly())
		if err != nil {
			return nil, oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID, "pattern", pattern).Wrapf(err, "match artifact pattern")
		}
		for _, match := range matches {
			relative := normalizeArtifactPath(match)
			if relative == "" {
				continue
			}
			if seen.Contains(relative) {
				continue
			}
			seen.Add(relative)
			info, err := fs.Stat(root, relative)
			if err != nil || info.IsDir() {
				continue
			}
			content, err := os.ReadFile(filepath.Join(workDir, filepath.FromSlash(relative)))
			if err != nil {
				return nil, oops.In("runner_agent").With("project_id", job.ProjectID, "job_id", job.ID, "artifact_path", relative).Wrapf(err, "read artifact")
			}
			files.Add(ArtifactFile{
				Name:        relative,
				FileName:    path.Base(relative),
				FilePath:    relative,
				ContentType: detectArtifactContentType(relative, content),
				Content:     content,
			})
		}
	}
	return files.Values(), nil
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
	switch {
	case strings.HasSuffix(strings.ToLower(fileName), ".txt"):
		return "text/plain"
	case strings.HasSuffix(strings.ToLower(fileName), ".json"):
		return "application/json"
	default:
		if len(content) > 0 {
			return http.DetectContentType(content)
		}
		return "application/octet-stream"
	}
}
