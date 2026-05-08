package ports

import (
	"context"
	"strconv"

	collectionlist "github.com/arcgolabs/collectionx/list"
	"path"
	"strings"
)

type ObjectStorage interface {
	SaveObject(ctx context.Context, key string, content []byte, contentType string) error
	SaveIssueAttachment(ctx context.Context, projectFullPath string, issueIID, attachmentID int64, fileName string, content []byte, contentType string) (string, error)
	SavePackageFile(ctx context.Context, projectFullPath, packageType, packageName, version string, fileID int64, fileName string, content []byte, contentType string) (string, error)
	SaveLFSObject(ctx context.Context, projectFullPath, oid string, content []byte) (string, error)
	SavePipelineArtifact(ctx context.Context, projectFullPath string, projectJobID, artifactID int64, fileName string, content []byte, contentType string) (string, error)
	Load(ctx context.Context, key string) ([]byte, error)
}

func BuildIssueStorageKey(projectFullPath string, issueIID, attachmentID int64, fileName string) string {
	return path.Join("issues", sanitizeNestedPath(projectFullPath), strconv.FormatInt(issueIID, 10), strconv.FormatInt(attachmentID, 10), sanitizeFileName(fileName))
}

func BuildIssueDraftStorageKey(projectFullPath, token, fileName string) string {
	return path.Join(BuildIssueDraftStoragePrefix(projectFullPath), sanitizePathSegment(token), sanitizeFileName(fileName))
}

func BuildIssueDraftStoragePrefix(projectFullPath string) string {
	return path.Join("issues", "drafts", sanitizeNestedPath(projectFullPath))
}

func BuildPackageStorageKey(projectFullPath, packageType, packageName, version string, fileID int64, fileName string) string {
	return path.Join("packages", sanitizeNestedPath(projectFullPath), sanitizePathSegment(packageType), sanitizePathSegment(packageName), sanitizePathSegment(version), strconv.FormatInt(fileID, 10), sanitizeFileName(fileName))
}

func BuildLFSStorageKey(projectFullPath, oid string) string {
	return path.Join("lfs", sanitizeNestedPath(projectFullPath), sanitizePathSegment(oid))
}

func BuildPipelineArtifactStorageKey(projectFullPath string, projectJobID, artifactID int64, fileName string) string {
	return path.Join("pipelines", sanitizeNestedPath(projectFullPath), "jobs", strconv.FormatInt(projectJobID, 10), "artifacts", strconv.FormatInt(artifactID, 10), sanitizeFileName(fileName))
}

func DetectContentType(fileName string) string {
	name := strings.ToLower(strings.TrimSpace(fileName))
	switch {
	case strings.HasSuffix(name, ".md"):
		return "text/markdown"
	case strings.HasSuffix(name, ".txt"):
		return "text/plain"
	case strings.HasSuffix(name, ".json"):
		return "application/json"
	case strings.HasSuffix(name, ".xml"):
		return "application/xml"
	case strings.HasSuffix(name, ".pom"):
		return "application/xml"
	case strings.HasSuffix(name, ".jar"):
		return "application/java-archive"
	case strings.HasSuffix(name, ".tgz"):
		return "application/gzip"
	case strings.HasSuffix(name, ".tar"):
		return "application/x-tar"
	case strings.HasSuffix(name, ".png"):
		return "image/png"
	case strings.HasSuffix(name, ".jpg") || strings.HasSuffix(name, ".jpeg"):
		return "image/jpeg"
	case strings.HasSuffix(name, ".gif"):
		return "image/gif"
	default:
		return "application/octet-stream"
	}
}

func normalizeStorageKey(key string) string {
	return strings.Trim(strings.ReplaceAll(key, "\\", "/"), "/")
}

func sanitizeNestedPath(value string) string {
	parts := strings.Split(normalizeStorageKey(value), "/")
	sanitized := collectionlist.FilterMapList(collectionlist.NewList(parts...), func(_ int, part string) (string, bool) {
		part = sanitizePathSegment(part)
		if part == "" {
			return "", false
		}
		return part, true
	}).Values()
	if len(sanitized) == 0 {
		return "unknown"
	}
	return path.Join(sanitized...)
}

func sanitizePathSegment(value string) string {
	trimmed := strings.TrimSpace(value)
	if trimmed == "" {
		return "unknown"
	}
	replacer := strings.NewReplacer("<", "_", ">", "_", ":", "_", "\"", "_", "|", "_", "?", "_", "*", "_", "\\", "_", "/", "_")
	trimmed = replacer.Replace(trimmed)
	trimmed = strings.Trim(trimmed, ". ")
	if trimmed == "" {
		return "unknown"
	}
	return trimmed
}

func sanitizeFileName(fileName string) string {
	trimmed := strings.TrimSpace(fileName)
	if trimmed == "" {
		return "blob.bin"
	}
	trimmed = strings.ReplaceAll(trimmed, "\\", "/")
	trimmed = path.Base(trimmed)
	trimmed = sanitizePathSegment(trimmed)
	if trimmed == "" || trimmed == "unknown" {
		return "blob.bin"
	}
	return trimmed
}
