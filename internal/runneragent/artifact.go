package runneragent

import (
	"encoding/json"
	"fmt"
	cidomain "github.com/DaiYuANg/gity/internal/domain/ci"
	"github.com/bmatcuk/doublestar/v4"
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
		return nil, fmt.Errorf("decode script job payload: %w", err)
	}
	if len(payload.Artifacts) == 0 {
		return nil, nil
	}
	var scriptResult ScriptResult
	if err := json.Unmarshal([]byte(strings.TrimSpace(result)), &scriptResult); err != nil {
		return nil, fmt.Errorf("decode script result: %w", err)
	}
	workDir := strings.TrimSpace(scriptResult.WorkDir)
	if workDir == "" {
		return nil, fmt.Errorf("script result does not include work_dir")
	}
	root := os.DirFS(workDir)
	files := make([]ArtifactFile, 0)
	seen := map[string]struct{}{}
	for _, pattern := range payload.Artifacts {
		matches, err := doublestar.Glob(root, normalizeArtifactPattern(pattern), doublestar.WithFilesOnly())
		if err != nil {
			return nil, fmt.Errorf("match artifact pattern %q: %w", pattern, err)
		}
		for _, match := range matches {
			relative := normalizeArtifactPath(match)
			if relative == "" {
				continue
			}
			if _, ok := seen[relative]; ok {
				continue
			}
			seen[relative] = struct{}{}
			info, err := fs.Stat(root, relative)
			if err != nil || info.IsDir() {
				continue
			}
			content, err := os.ReadFile(filepath.Join(workDir, filepath.FromSlash(relative)))
			if err != nil {
				return nil, fmt.Errorf("read artifact %q: %w", relative, err)
			}
			files = append(files, ArtifactFile{
				Name:        relative,
				FileName:    path.Base(relative),
				FilePath:    relative,
				ContentType: detectArtifactContentType(relative, content),
				Content:     content,
			})
		}
	}
	return files, nil
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
