package packageregistry

import (
	"context"

	"github.com/arcgolabs/httpx"
	"github.com/arcgolabs/mapper"
	packageregistryservice "github.com/lyonbrown4d/gity/internal/application/package_registry"
	projectservice "github.com/lyonbrown4d/gity/internal/application/project"
	infraauth "github.com/lyonbrown4d/gity/internal/infrastructure/auth"
	"github.com/lyonbrown4d/gity/internal/infrastructure/mapperx"
	"github.com/lyonbrown4d/gity/internal/interfaces/http_api"
)

type projectPackagesInput struct {
	ProjectID     int64  `path:"id"`
	Authorization string `header:"Authorization"`
}

type packageInput struct {
	ProjectID     int64  `path:"id"`
	PackageID     int64  `path:"package_id"`
	Authorization string `header:"Authorization"`
}

type packageFileInput struct {
	ProjectID     int64  `path:"id"`
	FileID        int64  `path:"file_id"`
	Authorization string `header:"Authorization"`
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
	authRuntime    *infraauth.Runtime
	mapper         *mapper.Mapper
}

func NewEndpoint(service *packageregistryservice.Service, projectService *projectservice.Service, authRuntime *infraauth.Runtime, requestMapper *mapper.Mapper) *Endpoint {
	return &Endpoint{service: service, projectService: projectService, authRuntime: authRuntime, mapper: mapperx.Ensure(requestMapper)}
}

func (e *Endpoint) EndpointSpec() httpx.EndpointSpec {
	return httpapi.EndpointSpec("/v1", "Package Registry", "Package Registry", "Project package registry APIs.")
}

func (e *Endpoint) Register(registrar httpx.Registrar) {
	service := e.service
	authRuntime := e.authRuntime
	projectScope := httpapi.ProjectScopeResolverFrom(e.projectService)

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
		input, err := mapperx.MapStrict[packageregistryservice.UploadFileInput](e.mapper, in.Body)
		if err != nil {
			return nil, err
		}
		item, err := service.UploadFile(ctx, in.ProjectID, input)
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
		httpapi.Get("/projects/{id}/packages", listPackages, httpapi.RequireProjectActionRoute[projectPackagesInput, packageOutput]("require_package_read", authRuntime, projectScope, infraauth.ProjectActionPackageRead)),
		httpapi.Get("/repos/{id}/packages", listPackages,
			httpapi.RequireProjectActionRoute[projectPackagesInput, packageOutput]("require_package_read", authRuntime, projectScope, infraauth.ProjectActionPackageRead),
			httpapi.DeprecatedRoute[projectPackagesInput, packageOutput]("Use GET /projects/{id}/packages instead."),
		),
	)

	e.registerProtocolRoutes(registrar, authRuntime, projectScope)

	httpapi.MustRegisterRoutes(registrar,
		httpapi.Get("/projects/{id}/packages/{package_id}", getPackage, httpapi.RequireProjectActionRoute[packageInput, packageOutput]("require_package_read", authRuntime, projectScope, infraauth.ProjectActionPackageRead)),
		httpapi.Get("/repos/{id}/packages/{package_id}", getPackage,
			httpapi.RequireProjectActionRoute[packageInput, packageOutput]("require_package_read", authRuntime, projectScope, infraauth.ProjectActionPackageRead),
			httpapi.DeprecatedRoute[packageInput, packageOutput]("Use GET /projects/{id}/packages/{package_id} instead."),
		),
		httpapi.Post("/projects/{id}/packages/files", uploadPackageFile, httpapi.RequireProjectActionRoute[uploadPackageFileInput, packageOutput]("require_package_write", authRuntime, projectScope, infraauth.ProjectActionPackageWrite)),
		httpapi.Post("/repos/{id}/packages/files", uploadPackageFile,
			httpapi.RequireProjectActionRoute[uploadPackageFileInput, packageOutput]("require_package_write", authRuntime, projectScope, infraauth.ProjectActionPackageWrite),
			httpapi.DeprecatedRoute[uploadPackageFileInput, packageOutput]("Use POST /projects/{id}/packages/files instead."),
		),
		httpapi.Get("/projects/{id}/packages/files/{file_id}", getPackageFile, httpapi.RequireProjectActionRoute[packageFileInput, packageOutput]("require_package_read", authRuntime, projectScope, infraauth.ProjectActionPackageRead)),
		httpapi.Get("/repos/{id}/packages/files/{file_id}", getPackageFile,
			httpapi.RequireProjectActionRoute[packageFileInput, packageOutput]("require_package_read", authRuntime, projectScope, infraauth.ProjectActionPackageRead),
			httpapi.DeprecatedRoute[packageFileInput, packageOutput]("Use GET /projects/{id}/packages/files/{file_id} instead."),
		),
	)
}

func (in projectPackagesInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in projectPackagesInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in packageInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in packageInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in packageFileInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in packageFileInput) ProjectIDValue() int64 {
	return in.ProjectID
}

func (in uploadPackageFileInput) AuthorizationHeader() string {
	return in.Authorization
}

func (in uploadPackageFileInput) ProjectIDValue() int64 {
	return in.ProjectID
}
