package packageregistry

import (
	"fmt"
	"sort"
	"strings"

	"github.com/arcgolabs/httpx"
	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	packageregistryservice "github.com/lyonbrown4d/gity/internal/application/package_registry"
	packagedomain "github.com/lyonbrown4d/gity/internal/domain/package_registry"
	"github.com/samber/oops"
)

type mavenCoordinate struct {
	PackageName string
	Version     string
	FilePath    string
}

func mavenCoordinateFromPath(filePath string) (mavenCoordinate, error) {
	normalized := strings.Trim(strings.ReplaceAll(filePath, "\\", "/"), "/")
	parts := strings.Split(normalized, "/")
	if len(parts) < 3 {
		return mavenCoordinate{}, apperror.BadRequest("maven package path is invalid", oops.In("package_registry").With("file_path", filePath).New("maven package path is invalid"))
	}
	artifact := parts[len(parts)-3]
	version := parts[len(parts)-2]
	groupParts := parts[:len(parts)-3]
	packageName := artifact
	if len(groupParts) > 0 {
		packageName = strings.Join(groupParts, ".") + ":" + artifact
	}
	return mavenCoordinate{PackageName: packageName, Version: version, FilePath: normalized}, nil
}

func normalizeTailPackageName(value httpx.PathTail) string {
	return strings.Trim(strings.ReplaceAll(value.String(), "\\", "/"), "/")
}

func attachmentContentType(attachment npmAttachment) string {
	if strings.TrimSpace(attachment.ContentType) != "" {
		return strings.TrimSpace(attachment.ContentType)
	}
	return "application/octet-stream"
}

func resolveNPMVersion(body npmPublishBody) string {
	if latest := strings.TrimSpace(body.DistTags["latest"]); latest != "" {
		return latest
	}
	keys := make([]string, 0, len(body.Versions))
	for key := range body.Versions {
		if strings.TrimSpace(key) != "" {
			keys = append(keys, strings.TrimSpace(key))
		}
	}
	sort.Strings(keys)
	if len(keys) == 0 {
		return ""
	}
	return keys[len(keys)-1]
}

func npmMetadata(projectID int64, packageName string, detail packageregistryservice.PackageDetail) map[string]any {
	versions := map[string]any{}
	versionNames := make([]string, 0, len(detail.Versions))
	for index := range detail.Versions {
		version := detail.Versions[index]
		versionName := version.Version.Version
		versionNames = append(versionNames, versionName)
		versions[versionName] = map[string]any{
			"name":    packageName,
			"version": versionName,
			"dist":    npmDist(projectID, version.Files),
		}
	}
	sort.Strings(versionNames)
	latest := ""
	if len(versionNames) > 0 {
		latest = versionNames[len(versionNames)-1]
	}
	return map[string]any{
		"name":      packageName,
		"dist-tags": map[string]string{"latest": latest},
		"versions":  versions,
	}
}

func npmDist(projectID int64, files []packagedomain.ProjectPackageFile) map[string]any {
	if len(files) == 0 {
		return map[string]any{}
	}
	fileRecord := files[0]
	for index := range files {
		candidate := files[index]
		if strings.HasSuffix(candidate.FileName, ".tgz") {
			fileRecord = candidate
			break
		}
	}
	return map[string]any{
		"tarball": packageDownloadURL(projectID, fileRecord.ID),
	}
}

func binaryResponse(blob packageregistryservice.PackageFileBlob) *packageBinaryOutput {
	contentType := strings.TrimSpace(blob.File.ContentType)
	if contentType == "" {
		contentType = "application/octet-stream"
	}
	fileName := strings.ReplaceAll(blob.File.FileName, `"`, "")
	return &packageBinaryOutput{
		ContentType:        contentType,
		ContentDisposition: fmt.Sprintf("attachment; filename=%q", fileName),
		Body:               httpx.StreamBytes(blob.Content),
	}
}

func htmlResponse(value string) *packageHTMLOutput {
	return &packageHTMLOutput{
		ContentType: "text/html; charset=utf-8",
		Body:        httpx.StreamReader(strings.NewReader(value)),
	}
}

func packageDownloadURL(projectID, fileID int64) string {
	return fmt.Sprintf("/api/v1/projects/%d/packages/files/%d/download", projectID, fileID)
}

func appendHTML(builder *strings.Builder, value string) {
	if builder == nil {
		return
	}
	if _, err := builder.WriteString(value); err != nil {
		return
	}
}

func appendHTMLf(builder *strings.Builder, format string, args ...any) {
	if builder == nil {
		return
	}
	if _, err := fmt.Fprintf(builder, format, args...); err != nil {
		return
	}
}
