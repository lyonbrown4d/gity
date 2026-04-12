package packageregistry

import (
	"context"

	"github.com/DaiYuANg/arcgo/httpx"
	packageregistryservice "github.com/DaiYuANg/gity/internal/service/packageregistry"
)

type packageInput struct {
	ProjectID int64 `path:"id"`
	PackageID int64 `path:"package_id"`
}

type packageFileInput struct {
	ProjectID int64 `path:"id"`
	FileID    int64 `path:"file_id"`
}

type uploadPackageFileInput struct {
	ProjectID int64                 `path:"id"`
	Body      uploadPackageFileBody `json:"body"`
}

type packageOutput struct {
	Body any `json:"body"`
}

type uploadPackageFileBody struct {
	Type          string `json:"type"`
	Name          string `json:"name"`
	Version       string `json:"version"`
	FileName      string `json:"file_name"`
	FilePath      string `json:"file_path"`
	ContentType   string `json:"content_type"`
	ContentBase64 string `json:"content_base64"`
}

func RegisterRoutes(server httpx.ServerRuntime, service *packageregistryservice.Service) {
	v1 := server.Group("/v1")

	httpx.MustGroupGet(v1, "/projects/{id}/packages", func(ctx context.Context, in *struct {
		ProjectID int64 `path:"id"`
	}) (*packageOutput, error) {
		items, err := service.ListPackages(ctx, in.ProjectID)
		if err != nil {
			return nil, err
		}
		return &packageOutput{Body: items}, nil
	})

	httpx.MustGroupGet(v1, "/projects/{id}/packages/{package_id}", func(ctx context.Context, in *packageInput) (*packageOutput, error) {
		item, err := service.GetPackage(ctx, in.ProjectID, in.PackageID)
		if err != nil {
			return nil, err
		}
		return &packageOutput{Body: item}, nil
	})

	httpx.MustGroupPost(v1, "/projects/{id}/packages/files", func(ctx context.Context, in *uploadPackageFileInput) (*packageOutput, error) {
		item, err := service.UploadFile(ctx, in.ProjectID, packageregistryservice.UploadFileInput{
			Type:          in.Body.Type,
			Name:          in.Body.Name,
			Version:       in.Body.Version,
			FileName:      in.Body.FileName,
			FilePath:      in.Body.FilePath,
			ContentType:   in.Body.ContentType,
			ContentBase64: in.Body.ContentBase64,
		})
		if err != nil {
			return nil, err
		}
		return &packageOutput{Body: item}, nil
	})

	httpx.MustGroupGet(v1, "/projects/{id}/packages/files/{file_id}", func(ctx context.Context, in *packageFileInput) (*packageOutput, error) {
		item, err := service.GetFileContent(ctx, in.ProjectID, in.FileID)
		if err != nil {
			return nil, err
		}
		return &packageOutput{Body: item}, nil
	})
}
