package packageregistry

import (
	"context"

	"github.com/DaiYuANg/gity/internal/httpapi"
	platformauth "github.com/DaiYuANg/gity/internal/platform/auth"
	packageregistryservice "github.com/DaiYuANg/gity/internal/service/packageregistry"
	projectservice "github.com/DaiYuANg/gity/internal/service/project"
	"github.com/arcgolabs/httpx"
)

type projectPackagesInput struct {
	ProjectID int64 `path:"id"`
}

type packageInput struct {
	ProjectID int64 `path:"id"`
	PackageID int64 `path:"package_id"`
}

type packageFileInput struct {
	ProjectID int64 `path:"id"`
	FileID    int64 `path:"file_id"`
}

type uploadPackageFileInput struct {
	ProjectID     int64                 `path:"id"`
	Authorization string                `header:"Authorization"`
	Body          uploadPackageFileBody `json:"body"`
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

type Endpoint struct {
	service        *packageregistryservice.Service
	projectService *projectservice.Service
	authRuntime    *platformauth.Runtime
}

func NewEndpoint(service *packageregistryservice.Service, projectService *projectservice.Service, authRuntime *platformauth.Runtime) *Endpoint {
	return &Endpoint{service: service, projectService: projectService, authRuntime: authRuntime}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Package Registry", "Package Registry", "Project package registry APIs.")
}

func RegisterRoutes(server httpx.ServerRuntime, service *packageregistryservice.Service, projectService *projectservice.Service, authRuntime *platformauth.Runtime) {
	server.RegisterOnly(NewEndpoint(service, projectService, authRuntime))
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	service := e.service
	authRuntime := e.authRuntime
	projectWrite := httpapi.ProjectScopeResolverFrom(e.projectService)

	listPackages := func(ctx context.Context, in *projectPackagesInput) (*packageOutput, error) {
		items, err := service.ListPackages(ctx, in.ProjectID)
		if err != nil {
			return nil, err
		}
		return &packageOutput{Body: items}, nil
	}

	getPackage := func(ctx context.Context, in *packageInput) (*packageOutput, error) {
		item, err := service.GetPackage(ctx, in.ProjectID, in.PackageID)
		if err != nil {
			return nil, err
		}
		return &packageOutput{Body: item}, nil
	}

	uploadPackageFile := func(ctx context.Context, in *uploadPackageFileInput) (*packageOutput, error) {
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
	}

	getPackageFile := func(ctx context.Context, in *packageFileInput) (*packageOutput, error) {
		item, err := service.GetFileContent(ctx, in.ProjectID, in.FileID)
		if err != nil {
			return nil, err
		}
		return &packageOutput{Body: item}, nil
	}

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/packages", listPackages),
		httpapi.Get("/repos/{id}/packages", listPackages, httpapi.DeprecatedRoute[projectPackagesInput, packageOutput]("Use GET /projects/{id}/packages instead.")),
		httpapi.Get("/projects/{id}/packages/{package_id}", getPackage),
		httpapi.Get("/repos/{id}/packages/{package_id}", getPackage, httpapi.DeprecatedRoute[packageInput, packageOutput]("Use GET /projects/{id}/packages/{package_id} instead.")),
		httpapi.Post("/projects/{id}/packages/files", uploadPackageFile, httpapi.RequireProjectWriteRoute[uploadPackageFileInput, packageOutput](authRuntime, projectWrite)),
		httpapi.Post("/repos/{id}/packages/files", uploadPackageFile,
			httpapi.RequireProjectWriteRoute[uploadPackageFileInput, packageOutput](authRuntime, projectWrite),
			httpapi.DeprecatedRoute[uploadPackageFileInput, packageOutput]("Use POST /projects/{id}/packages/files instead."),
		),
		httpapi.Get("/projects/{id}/packages/files/{file_id}", getPackageFile),
		httpapi.Get("/repos/{id}/packages/files/{file_id}", getPackageFile, httpapi.DeprecatedRoute[packageFileInput, packageOutput]("Use GET /projects/{id}/packages/files/{file_id} instead.")),
	)
}

func (in uploadPackageFileInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in uploadPackageFileInput) ProjectIDValue() int64 {
	return in.ProjectID
}
