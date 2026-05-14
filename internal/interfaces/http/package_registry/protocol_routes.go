package packageregistry

import (
	"context"
	"encoding/base64"
	"fmt"
	"html"
	"io"
	"path"
	"strings"

	"github.com/arcgolabs/httpx"
	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	packageregistryservice "github.com/lyonbrown4d/gity/internal/application/package_registry"
	packagedomain "github.com/lyonbrown4d/gity/internal/domain/package_registry"
	"github.com/samber/oops"
)

func (e *Endpoint) uploadGenericPackageFile(ctx context.Context, in *protocolPackageFileInput) (*packageOutput, error) {
	item, err := e.uploadProtocolFile(ctx, "generic", in.ProjectID, in.PackageName, in.PackageVersion, in.FileName.String(), in.ContentType, in.Payload)
	if err != nil {
		return nil, err
	}
	return &packageOutput{Body: item}, nil
}

func (e *Endpoint) downloadGenericPackageFile(ctx context.Context, in *protocolPackageDownloadInput) (*packageBinaryOutput, error) {
	return e.downloadProtocolFile(ctx, in.ProjectID, "generic", in.PackageName, in.PackageVersion, in.FileName.String())
}

func (e *Endpoint) uploadNuGetPackageFile(ctx context.Context, in *protocolPackageFileInput) (*packageOutput, error) {
	item, err := e.uploadProtocolFile(ctx, "nuget", in.ProjectID, in.PackageName, in.PackageVersion, in.FileName.String(), in.ContentType, in.Payload)
	if err != nil {
		return nil, err
	}
	return &packageOutput{Body: item}, nil
}

func (e *Endpoint) downloadNuGetPackageFile(ctx context.Context, in *protocolPackageDownloadInput) (*packageBinaryOutput, error) {
	return e.downloadProtocolFile(ctx, in.ProjectID, "nuget", in.PackageName, in.PackageVersion, in.FileName.String())
}

func (e *Endpoint) nugetServiceIndex(_ context.Context, in *pypiIndexInput) (*packageOutput, error) {
	base := fmt.Sprintf("/api/v1/projects/%d/packages/nuget", in.ProjectID)
	return &packageOutput{Body: map[string]any{
		"version": "3.0.0",
		"resources": []map[string]string{
			{"@id": base, "@type": "PackagePublish/2.0.0"},
			{"@id": base + "/download", "@type": "PackageBaseAddress/3.0.0"},
		},
	}}, nil
}

func (e *Endpoint) uploadPyPIPackageFile(ctx context.Context, in *protocolPackageFileInput) (*packageOutput, error) {
	item, err := e.uploadProtocolFile(ctx, "pypi", in.ProjectID, in.PackageName, in.PackageVersion, in.FileName.String(), in.ContentType, in.Payload)
	if err != nil {
		return nil, err
	}
	return &packageOutput{Body: item}, nil
}

func (e *Endpoint) pypiSimpleIndex(ctx context.Context, in *pypiIndexInput) (*packageHTMLOutput, error) {
	items, err := e.service.ListPackages(ctx, in.ProjectID)
	if err != nil {
		return nil, err
	}
	var builder strings.Builder
	appendHTML(&builder, "<!doctype html><html><body>\n")
	for index := range items {
		item := items[index]
		if strings.EqualFold(item.Type, "pypi") {
			appendHTMLf(&builder, `<a href="./%s/">%s</a><br/>`+"\n", html.EscapeString(item.Name), html.EscapeString(item.Name))
		}
	}
	appendHTML(&builder, "</body></html>\n")
	return htmlResponse(builder.String()), nil
}

func (e *Endpoint) pypiSimplePackage(ctx context.Context, in *pypiPackageInput) (*packageHTMLOutput, error) {
	detail, err := e.service.GetPackageByTypeAndName(ctx, in.ProjectID, "pypi", in.PackageName)
	if err != nil {
		return nil, err
	}
	var builder strings.Builder
	appendHTML(&builder, "<!doctype html><html><body>\n")
	for versionIndex := range detail.Versions {
		files := detail.Versions[versionIndex].Files
		for fileIndex := range files {
			fileRecord := files[fileIndex]
			appendHTMLf(&builder, `<a href="%s">%s</a><br/>`+"\n", packageDownloadURL(in.ProjectID, fileRecord.ID), html.EscapeString(fileRecord.FileName))
		}
	}
	appendHTML(&builder, "</body></html>\n")
	return htmlResponse(builder.String()), nil
}

func (e *Endpoint) uploadMavenPackageFile(ctx context.Context, in *mavenPackageFileInput) (*packageOutput, error) {
	coordinate, err := mavenCoordinateFromPath(in.FilePath.String())
	if err != nil {
		return nil, err
	}
	item, err := e.uploadProtocolFile(ctx, "maven", in.ProjectID, coordinate.PackageName, coordinate.Version, coordinate.FilePath, in.ContentType, in.Payload)
	if err != nil {
		return nil, err
	}
	return &packageOutput{Body: item}, nil
}

func (e *Endpoint) downloadMavenPackageFile(ctx context.Context, in *mavenPackageDownloadInput) (*packageBinaryOutput, error) {
	coordinate, err := mavenCoordinateFromPath(in.FilePath.String())
	if err != nil {
		return nil, err
	}
	return e.downloadProtocolFile(ctx, in.ProjectID, "maven", coordinate.PackageName, coordinate.Version, coordinate.FilePath)
}

func (e *Endpoint) getNPMPackageMetadata(ctx context.Context, in *npmPackageInput) (*packageOutput, error) {
	packageName := normalizeTailPackageName(in.PackageName)
	detail, err := e.service.GetPackageByTypeAndName(ctx, in.ProjectID, "npm", packageName)
	if err != nil {
		return nil, err
	}
	return &packageOutput{Body: npmMetadata(in.ProjectID, packageName, detail)}, nil
}

func (e *Endpoint) publishNPMPackage(ctx context.Context, in *npmPublishInput) (*packageOutput, error) {
	packageName := strings.TrimSpace(in.Body.Name)
	if packageName == "" {
		packageName = normalizeTailPackageName(in.PackageName)
	}
	version := resolveNPMVersion(in.Body)
	if packageName == "" || version == "" {
		return nil, apperror.BadRequest("npm package name and version are required", oops.In("package_registry").New("npm package name and version are required"))
	}
	if len(in.Body.Attachments) == 0 {
		return nil, apperror.BadRequest("npm package attachment is required", oops.In("package_registry").With("package", packageName, "version", version).New("npm package attachment is required"))
	}
	var uploaded packagedomain.ProjectPackageFile
	for fileName, attachment := range in.Body.Attachments {
		content, err := base64.StdEncoding.DecodeString(strings.TrimSpace(attachment.Data))
		if err != nil {
			return nil, apperror.BadRequest("npm package attachment is invalid", oops.In("package_registry").With("package", packageName, "version", version, "file_name", fileName).Wrapf(err, "decode npm attachment"))
		}
		uploaded, err = e.service.UploadRawFile(ctx, in.ProjectID, packageregistryservice.UploadRawFileInput{
			Type:        "npm",
			Name:        packageName,
			Version:     version,
			FileName:    path.Base(fileName),
			FilePath:    fileName,
			ContentType: attachmentContentType(attachment),
			Content:     content,
		})
		if err != nil {
			return nil, err
		}
	}
	detail, err := e.service.GetPackageByTypeAndName(ctx, in.ProjectID, "npm", packageName)
	if err != nil {
		return nil, err
	}
	body := npmMetadata(in.ProjectID, packageName, detail)
	body["uploaded_file_id"] = uploaded.ID
	return &packageOutput{Body: body}, nil
}

func (e *Endpoint) downloadPackageFile(ctx context.Context, in *packageFileDownloadInput) (*packageBinaryOutput, error) {
	blob, err := e.service.GetFileBlob(ctx, in.ProjectID, in.FileID)
	if err != nil {
		return nil, err
	}
	return binaryResponse(blob), nil
}

func (e *Endpoint) uploadProtocolFile(ctx context.Context, packageType string, projectID int64, packageName, version, filePath, contentType string, payload httpx.RequestStream) (packagedomain.ProjectPackageFile, error) {
	content, err := io.ReadAll(payload.Reader())
	if err != nil {
		return packagedomain.ProjectPackageFile{}, oops.In("package_registry").With("project_id", projectID, "type", packageType, "name", packageName, "version", version, "file_path", filePath).Wrapf(err, "read package request body")
	}
	fileName := path.Base(strings.Trim(filePath, "/"))
	return e.service.UploadRawFile(ctx, projectID, packageregistryservice.UploadRawFileInput{
		Type:        packageType,
		Name:        packageName,
		Version:     version,
		FileName:    fileName,
		FilePath:    filePath,
		ContentType: contentType,
		Content:     content,
	})
}

func (e *Endpoint) downloadProtocolFile(ctx context.Context, projectID int64, packageType, packageName, version, filePath string) (*packageBinaryOutput, error) {
	blob, err := e.service.GetFileByCoordinate(ctx, projectID, packageType, packageName, version, filePath)
	if err != nil {
		return nil, err
	}
	return binaryResponse(blob), nil
}
