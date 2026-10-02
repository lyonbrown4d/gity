package packageregistry_test

import (
	"context"
	"io"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/arcgolabs/httpx"
	httpxfiber "github.com/arcgolabs/httpx/adapter/fiber"
	"github.com/gofiber/fiber/v3"
	packageregistryservice "github.com/lyonbrown4d/gity/internal/application/package_registry"
	projectservice "github.com/lyonbrown4d/gity/internal/application/project"
	packagedomain "github.com/lyonbrown4d/gity/internal/domain/package_registry"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	packageregistry "github.com/lyonbrown4d/gity/internal/interfaces/http/package_registry"
)

type conditionalDownloadCase struct {
	name              string
	packageType       string
	packageName       string
	fileName          string
	filePath          string
	url               string
	conditionalHeader string
}

func TestPackageFileDownloadsSupportConditionalRequestsWithoutLoadingBlob(t *testing.T) {
	modified := time.Date(2026, time.October, 2, 9, 8, 7, 456000000, time.UTC)
	tests := []conditionalDownloadCase{
		{
			name:              "generic If-Modified-Since",
			packageType:       "generic",
			packageName:       "widgets",
			fileName:          "widget.bin",
			filePath:          "widget.bin",
			url:               "/v1/projects/1/packages/generic/widgets/1.0.0/widget.bin",
			conditionalHeader: "If-Modified-Since",
		},
		{
			name:              "NuGet If-None-Match",
			packageType:       "nuget",
			packageName:       "Widgets",
			fileName:          "Widgets.1.0.0.nupkg",
			filePath:          "Widgets.1.0.0.nupkg",
			url:               "/v1/projects/1/packages/nuget/Widgets/1.0.0/Widgets.1.0.0.nupkg",
			conditionalHeader: "If-None-Match",
		},
		{
			name:              "Maven If-None-Match",
			packageType:       "maven",
			packageName:       "com.acme:widgets",
			fileName:          "widgets-1.0.0.jar",
			filePath:          "com/acme/widgets/1.0.0/widgets-1.0.0.jar",
			url:               "/v1/projects/1/packages/maven/com/acme/widgets/1.0.0/widgets-1.0.0.jar",
			conditionalHeader: "If-None-Match",
		},
		{
			name:              "PyPI file link If-None-Match",
			packageType:       "pypi",
			packageName:       "widgets",
			fileName:          "widgets-1.0.0.whl",
			filePath:          "widgets-1.0.0.whl",
			url:               "/v1/projects/1/packages/files/44/download",
			conditionalHeader: "If-None-Match",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			testConditionalDownload(t, test, modified)
		})
	}
}

func testConditionalDownload(t *testing.T, test conditionalDownloadCase, modified time.Time) {
	t.Helper()
	app, storage := newConditionalPackageServer(t, conditionalPackageFixture{
		packageType: test.packageType,
		packageName: test.packageName,
		fileName:    test.fileName,
		filePath:    test.filePath,
		modified:    modified,
	})
	etag, lastModified := assertInitialDownload(t, app, storage, test.url, modified)
	conditionalValue := etag
	if test.conditionalHeader == "If-Modified-Since" {
		conditionalValue = lastModified
	}
	assertNotModifiedDownload(t, app, storage, test.url, test.conditionalHeader, conditionalValue, etag, lastModified)
	assertPartialDownload(t, app, test.url)
	assertUnsatisfiableRange(t, app, test.url)
	assertIfRangeDownload(t, app, test.url, etag, lastModified)
}

func assertInitialDownload(t *testing.T, app *fiber.App, storage *conditionalObjectStorage, url string, modified time.Time) (string, string) {
	t.Helper()
	response := packageRequest(t, app, url, "", "")
	if response.status != http.StatusOK || response.body != "package-binary" {
		t.Fatalf("initial response: status=%d body=%q", response.status, response.body)
	}
	etag := response.header.Get("ETag")
	if etag == "" {
		t.Fatal("initial response did not include ETag")
	}
	lastModified := response.header.Get("Last-Modified")
	if want := modified.Truncate(time.Second).Format(http.TimeFormat); lastModified != want {
		t.Fatalf("Last-Modified = %q, want %q", lastModified, want)
	}
	if got := storage.loadCount(); got != 1 {
		t.Fatalf("initial request loaded blob %d times", got)
	}
	return etag, lastModified
}

func assertNotModifiedDownload(t *testing.T, app *fiber.App, storage *conditionalObjectStorage, url, header, value, etag, lastModified string) {
	t.Helper()
	response := packageRequest(t, app, url, header, value)
	if response.status != http.StatusNotModified || response.body != "" {
		t.Fatalf("conditional response: status=%d body=%q", response.status, response.body)
	}
	if got := storage.loadCount(); got != 1 {
		t.Fatalf("conditional request loaded blob; total loads=%d", got)
	}
	if got := response.header.Get("ETag"); got != etag {
		t.Fatalf("304 ETag = %q, want %q", got, etag)
	}
	if got := response.header.Get("Last-Modified"); got != lastModified {
		t.Fatalf("304 Last-Modified = %q, want %q", got, lastModified)
	}
}

func assertPartialDownload(t *testing.T, app *fiber.App, url string) {
	t.Helper()
	response := packageRequest(t, app, url, "Range", "bytes=0-6")
	if response.status != http.StatusPartialContent || response.body != "package" {
		t.Fatalf("range response: status=%d body=%q", response.status, response.body)
	}
	if got, want := response.header.Get("Content-Range"), "bytes 0-6/14"; got != want {
		t.Fatalf("Content-Range = %q, want %q", got, want)
	}
}

func assertUnsatisfiableRange(t *testing.T, app *fiber.App, url string) {
	t.Helper()
	response := packageRequest(t, app, url, "Range", "bytes=99-")
	if response.status != http.StatusRequestedRangeNotSatisfiable || response.body != "" {
		t.Fatalf("unsatisfiable range response: status=%d body=%q", response.status, response.body)
	}
	if got, want := response.header.Get("Content-Range"), "bytes */14"; got != want {
		t.Fatalf("416 Content-Range = %q, want %q", got, want)
	}
}

func assertIfRangeDownload(t *testing.T, app *fiber.App, url, etag, lastModified string) {
	t.Helper()
	assertIfRangeStatus(t, app, url, etag, http.StatusPartialContent)
	assertIfRangeStatus(t, app, url, `"different"`, http.StatusOK)
	assertIfRangeStatus(t, app, url, lastModified, http.StatusPartialContent)
	assertIfRangeStatus(t, app, url, time.Date(2026, time.October, 2, 9, 8, 6, 0, time.UTC).Format(http.TimeFormat), http.StatusOK)
}

func assertIfRangeStatus(t *testing.T, app *fiber.App, url, ifRange string, wantStatus int) {
	t.Helper()
	response := packageRequestWithHeaders(t, app, url, http.Header{
		"Range":    []string{"bytes=0-6"},
		"If-Range": []string{ifRange},
	})
	if response.status != wantStatus {
		t.Fatalf("If-Range %q status = %d, want %d", ifRange, response.status, wantStatus)
	}
	if wantStatus == http.StatusOK && (response.body != "package-binary" || response.header.Get("Content-Range") != "") {
		t.Fatalf("If-Range mismatch returned body=%q Content-Range=%q", response.body, response.header.Get("Content-Range"))
	}
	if wantStatus == http.StatusPartialContent && (response.body != "package" || response.header.Get("Content-Range") != "bytes 0-6/14") {
		t.Fatalf("If-Range match returned body=%q Content-Range=%q", response.body, response.header.Get("Content-Range"))
	}
}

type conditionalPackageFixture struct {
	packageType string
	packageName string
	fileName    string
	filePath    string
	modified    time.Time
}

func newConditionalPackageServer(t *testing.T, fixture conditionalPackageFixture) (*fiber.App, *conditionalObjectStorage) {
	t.Helper()
	project := projectdomain.Project{ID: 1, Visibility: "public"}
	pkg := packagedomain.ProjectPackage{ID: 22, ProjectID: project.ID, Type: fixture.packageType, Name: fixture.packageName}
	version := packagedomain.ProjectPackageVersion{ID: 33, ProjectPackageID: pkg.ID, Version: "1.0.0"}
	file := packagedomain.ProjectPackageFile{
		ID:                      44,
		ProjectPackageVersionID: version.ID,
		FileName:                fixture.fileName,
		FilePath:                fixture.filePath,
		ContentType:             "application/octet-stream",
		ByteSize:                int64(len("package-binary")),
		StorageKey:              "packages/test/44/" + fixture.fileName,
		CreatedAt:               fixture.modified.Add(-time.Minute),
		UpdatedAt:               fixture.modified,
	}
	storage := &conditionalObjectStorage{content: []byte("package-binary")}
	service := packageregistryservice.NewService(
		&conditionalProjectRepository{project: project},
		&conditionalPackageRepository{pkg: pkg},
		&conditionalVersionRepository{version: version},
		&conditionalFileRepository{file: file},
		storage,
	)
	projectService := projectservice.NewService(slog.Default(), &conditionalProjectRepository{project: project}, nil, nil, nil, nil)
	endpoint := packageregistry.NewEndpoint(service, projectService, nil, nil)
	app := fiber.New()
	adapter := httpxfiber.New(app)
	server := httpx.New(httpx.WithAdapter(adapter))
	server.RegisterOnly(endpoint)
	return app, storage
}

type packageResponse struct {
	status int
	header http.Header
	body   string
}

func packageRequest(t *testing.T, app *fiber.App, url, header, value string) packageResponse {
	t.Helper()
	headers := http.Header{}
	if header != "" {
		headers.Set(header, value)
	}
	return packageRequestWithHeaders(t, app, url, headers)
}

func packageRequestWithHeaders(t *testing.T, app *fiber.App, url string, headers http.Header) packageResponse {
	t.Helper()
	request := httptest.NewRequestWithContext(context.Background(), http.MethodGet, url, http.NoBody)
	for name, values := range headers {
		request.Header[name] = append([]string(nil), values...)
	}
	response, err := app.Test(request, fiber.TestConfig{Timeout: 0})
	if err != nil {
		t.Fatalf("execute package request: %v", err)
	}
	body, readErr := io.ReadAll(response.Body)
	closeErr := response.Body.Close()
	if readErr != nil {
		t.Fatalf("read response body: %v (close: %v)", readErr, closeErr)
	}
	if closeErr != nil {
		t.Fatalf("close response body: %v", closeErr)
	}
	return packageResponse{status: response.StatusCode, header: response.Header, body: string(body)}
}
