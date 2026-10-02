package packageregistry

import (
	"context"
	"fmt"
	"net/http"
	"sort"
	"strings"
	"time"

	"github.com/arcgolabs/httpx"
	"github.com/danielgtaylor/huma/v2"
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

func binaryResponse(blob packageregistryservice.PackageFileBlob, byteRange httpx.ByteRange, ifRange string) *packageBinaryOutput {
	contentType := strings.TrimSpace(blob.File.ContentType)
	if contentType == "" {
		contentType = "application/octet-stream"
	}
	fileName := strings.ReplaceAll(blob.File.FileName, `"`, "")
	response := &packageBinaryOutput{
		Status:             http.StatusOK,
		ContentType:        contentType,
		ContentDisposition: fmt.Sprintf("attachment; filename=%q", fileName),
		ETag:               formatPackageETag(blob.ETag),
		LastModified:       blob.LastModified.Format(http.TimeFormat),
		AcceptRanges:       "bytes",
		ContentLength:      int64(len(blob.Content)),
		Body:               blob.Content,
	}
	if byteRange.IsZero() || !packageIfRangeMatches(ifRange, blob.ETag, blob.LastModified) {
		return response
	}

	size := int64(len(blob.Content))
	start, end, ok := byteRange.Bounds(size)
	if !ok {
		response.Status = http.StatusRequestedRangeNotSatisfiable
		response.ContentRange = fmt.Sprintf("bytes */%d", size)
		response.ContentLength = 0
		response.Body = []byte{}
		return response
	}

	response.Status = http.StatusPartialContent
	response.ContentRange = fmt.Sprintf("bytes %d-%d/%d", start, end, size)
	response.ContentLength = end - start + 1
	response.Body = blob.Content[start : end+1]
	return response
}

func packageIfRangeMatches(value, etag string, modified time.Time) bool {
	trimmed := strings.TrimSpace(value)
	if trimmed == "" {
		return true
	}
	if strings.HasPrefix(trimmed, `W/`) {
		return false
	}
	if strings.HasPrefix(trimmed, `"`) {
		return trimmed == formatPackageETag(etag)
	}
	parsed, err := http.ParseTime(trimmed)
	if err != nil {
		return false
	}
	return !modified.After(parsed)
}

func formatPackageETag(etag string) string {
	return `"` + etag + `"`
}

type packageConditionalParamsGetter[I any] func(*I) *httpx.ConditionalParams

func packageConditionalReadPolicy[I, O any](paramsGetter packageConditionalParamsGetter[I], stateGetter httpx.ConditionalStateGetter[I]) httpx.RoutePolicy[I, O] {
	return httpx.RoutePolicy[I, O]{
		Name:      "conditional",
		Operation: httpx.OperationConditionalRead(),
		Wrap:      packageConditionalReadWrapper[I, O](paramsGetter, stateGetter),
	}
}

func packageConditionalReadWrapper[I, O any](paramsGetter packageConditionalParamsGetter[I], stateGetter httpx.ConditionalStateGetter[I]) func(httpx.TypedHandler[I, O]) httpx.TypedHandler[I, O] {
	return func(next httpx.TypedHandler[I, O]) httpx.TypedHandler[I, O] {
		if next == nil || paramsGetter == nil || stateGetter == nil {
			return next
		}
		return func(ctx context.Context, input *I) (*O, error) {
			return executePackageConditionalRead(ctx, input, next, paramsGetter, stateGetter)
		}
	}
}

func executePackageConditionalRead[I, O any](ctx context.Context, input *I, next httpx.TypedHandler[I, O], paramsGetter packageConditionalParamsGetter[I], stateGetter httpx.ConditionalStateGetter[I]) (*O, error) {
	params := paramsGetter(input)
	if params == nil || !params.HasConditionalParams() {
		return next(ctx, input)
	}
	etag, modified, err := stateGetter(ctx, input)
	if err != nil {
		return nil, err
	}
	if err := params.PreconditionFailed(etag, modified); err != nil {
		conditionalErr := huma.ErrorWithHeaders(err, packageValidatorHeaders(etag, modified))
		return nil, oops.In("http.package_registry").Wrapf(conditionalErr, "evaluate conditional package request")
	}
	return next(ctx, input)
}

func packageValidatorHeaders(etag string, modified time.Time) http.Header {
	headers := http.Header{}
	if etag != "" {
		headers.Set("ETag", formatPackageETag(etag))
	}
	if !modified.IsZero() {
		headers.Set("Last-Modified", modified.Format(http.TimeFormat))
	}
	return headers
}

func protocolPackageConditionalParams(input *protocolPackageDownloadInput) *httpx.ConditionalParams {
	if input == nil {
		return nil
	}
	return &input.ConditionalParams
}

func mavenPackageConditionalParams(input *mavenPackageDownloadInput) *httpx.ConditionalParams {
	if input == nil {
		return nil
	}
	return &input.ConditionalParams
}

func packageFileConditionalParams(input *packageFileDownloadInput) *httpx.ConditionalParams {
	if input == nil {
		return nil
	}
	return &input.ConditionalParams
}

func operationPackageBinaryResponse() httpx.OperationOption {
	binary := httpx.OperationBinaryResponse("application/octet-stream")
	return func(operation *huma.Operation) {
		binary(operation)
		if operation == nil {
			return
		}
		if operation.Responses == nil {
			operation.Responses = map[string]*huma.Response{}
		}
		partial := &huma.Response{Description: http.StatusText(http.StatusPartialContent)}
		if success := operation.Responses["200"]; success != nil {
			partial.Content = success.Content
		}
		operation.Responses["206"] = partial
		operation.Responses["416"] = &huma.Response{Description: http.StatusText(http.StatusRequestedRangeNotSatisfiable)}
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
