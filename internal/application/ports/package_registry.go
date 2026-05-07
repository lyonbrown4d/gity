package ports

import (
	"context"

	packagedomain "github.com/DaiYuANg/gity/internal/domain/package_registry"
	collectionx "github.com/arcgolabs/collectionx/list"
)

type ProjectPackageRepository interface {
	ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[packagedomain.ProjectPackage], error)
	GetByID(ctx context.Context, id int64) (packagedomain.ProjectPackage, error)
	GetByProjectTypeAndName(ctx context.Context, projectID int64, packageType string, name string) (packagedomain.ProjectPackage, error)
	Create(ctx context.Context, input CreateProjectPackageInput) (packagedomain.ProjectPackage, error)
}

type ProjectPackageVersionRepository interface {
	ListByPackageID(ctx context.Context, packageID int64) (*collectionx.List[packagedomain.ProjectPackageVersion], error)
	GetByID(ctx context.Context, id int64) (packagedomain.ProjectPackageVersion, error)
	GetByPackageAndVersion(ctx context.Context, packageID int64, version string) (packagedomain.ProjectPackageVersion, error)
	Create(ctx context.Context, input CreateProjectPackageVersionInput) (packagedomain.ProjectPackageVersion, error)
}

type ProjectPackageFileRepository interface {
	ListByVersionID(ctx context.Context, versionID int64) (*collectionx.List[packagedomain.ProjectPackageFile], error)
	ListByVersionIDs(ctx context.Context, versionIDs ...int64) (*collectionx.List[packagedomain.ProjectPackageFile], error)
	GetByID(ctx context.Context, id int64) (packagedomain.ProjectPackageFile, error)
	Create(ctx context.Context, input CreateProjectPackageFileInput) (packagedomain.ProjectPackageFile, error)
	MarkStored(ctx context.Context, fileID int64, input StoreProjectPackageFileInput) error
	DeleteByID(ctx context.Context, fileID int64) error
}

type CreateProjectPackageInput struct {
	ProjectID int64
	Type      string
	Name      string
}

type CreateProjectPackageVersionInput struct {
	ProjectPackageID int64
	Version          string
	Status           string
}

type CreateProjectPackageFileInput struct {
	ProjectPackageVersionID int64
	FileName                string
	FilePath                string
	ContentType             string
}

type StoreProjectPackageFileInput struct {
	ContentType string
	ByteSize    int64
	StorageKey  string
}
