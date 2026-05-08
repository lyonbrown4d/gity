package ports

import (
	"context"
	"path"
	"strconv"
	"strings"

	collectionlist "github.com/arcgolabs/collectionx/list"
	mappingx "github.com/arcgolabs/collectionx/mapping"
)

type ObjectStorage interface {
	SaveObject(ctx context.Context, key string, content []byte, contentType string) error
	SaveIssueAttachment(ctx context.Context, projectFullPath string, issueIID, attachmentID int64, fileName string, content []byte, contentType string) (string, error)
	SavePackageFile(ctx context.Context, projectFullPath, packageType, packageName, version string, fileID int64, fileName string, content []byte, contentType string) (string, error)
	SaveLFSObject(ctx context.Context, projectFullPath, oid string, content []byte) (string, error)
	SavePipelineArtifact(ctx context.Context, projectFullPath string, projectJobID, artifactID int64, fileName string, content []byte, contentType string) (string, error)
	Load(ctx context.Context, key string) ([]byte, error)
}

var contentTypesByExtension = mappingx.NewMapFrom(map[string]string{
	".md":   "text/markdown",
	".txt":  "text/plain",
	".json": "application/json",
	".xml":  "application/xml",
	".pom":  "application/xml",
	".jar":  "application/java-archive",
	".tgz":  "application/gzip",
	".tar":  "application/x-tar",
	".png":  "image/png",
	".jpg":  "image/jpeg",
	".jpeg": "image/jpeg",
	".gif":  "image/gif",
})

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
	extension := strings.ToLower(path.Ext(strings.TrimSpace(fileName)))
	if contentType, ok := contentTypesByExtension.Get(extension); ok {
		return contentType
	}
	return "application/octet-stream"
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
