package packageregistry_test

import (
	"context"
	"sync"
	"time"

	collectionlist "github.com/arcgolabs/collectionx/list"
	storageports "github.com/lyonbrown4d/gity/internal/application/ports"
	organizationdomain "github.com/lyonbrown4d/gity/internal/domain/organization"
	packagedomain "github.com/lyonbrown4d/gity/internal/domain/package_registry"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
)

type conditionalProjectRepository struct {
	project projectdomain.Project
}

func (r *conditionalProjectRepository) List(context.Context, *int64) (*collectionlist.List[projectdomain.Project], error) {
	return collectionlist.NewList(r.project), nil
}
func (r *conditionalProjectRepository) GetByID(context.Context, int64) (projectdomain.Project, error) {
	return r.project, nil
}
func (r *conditionalProjectRepository) GetIncludingDeletedByID(context.Context, int64) (projectdomain.Project, error) {
	return r.project, nil
}
func (r *conditionalProjectRepository) GetByFullPath(context.Context, string) (projectdomain.Project, error) {
	return r.project, nil
}
func (r *conditionalProjectRepository) Create(context.Context, storageports.CreateProjectInput, organizationdomain.Organization) (projectdomain.Project, error) {
	return r.project, nil
}
func (r *conditionalProjectRepository) MarkPendingDeleteByID(context.Context, int64, time.Time) error {
	return nil
}
func (r *conditionalProjectRepository) DeleteByID(context.Context, int64) error { return nil }

type conditionalPackageRepository struct {
	pkg packagedomain.ProjectPackage
}

func (r *conditionalPackageRepository) ListByProjectID(context.Context, int64) (*collectionlist.List[packagedomain.ProjectPackage], error) {
	return collectionlist.NewList(r.pkg), nil
}
func (r *conditionalPackageRepository) GetByID(context.Context, int64) (packagedomain.ProjectPackage, error) {
	return r.pkg, nil
}
func (r *conditionalPackageRepository) GetByProjectTypeAndName(_ context.Context, projectID int64, packageType, name string) (packagedomain.ProjectPackage, error) {
	pkg := r.pkg
	pkg.ProjectID = projectID
	pkg.Type = packageType
	pkg.Name = name
	return pkg, nil
}
func (r *conditionalPackageRepository) Create(context.Context, storageports.CreateProjectPackageInput) (packagedomain.ProjectPackage, error) {
	return r.pkg, nil
}

type conditionalVersionRepository struct {
	version packagedomain.ProjectPackageVersion
}

func (r *conditionalVersionRepository) ListByPackageID(context.Context, int64) (*collectionlist.List[packagedomain.ProjectPackageVersion], error) {
	return collectionlist.NewList(r.version), nil
}
func (r *conditionalVersionRepository) GetByID(context.Context, int64) (packagedomain.ProjectPackageVersion, error) {
	return r.version, nil
}
func (r *conditionalVersionRepository) GetByPackageAndVersion(context.Context, int64, string) (packagedomain.ProjectPackageVersion, error) {
	return r.version, nil
}
func (r *conditionalVersionRepository) Create(context.Context, storageports.CreateProjectPackageVersionInput) (packagedomain.ProjectPackageVersion, error) {
	return r.version, nil
}

type conditionalFileRepository struct {
	file packagedomain.ProjectPackageFile
}

func (r *conditionalFileRepository) ListByVersionID(context.Context, int64) (*collectionlist.List[packagedomain.ProjectPackageFile], error) {
	return collectionlist.NewList(r.file), nil
}
func (r *conditionalFileRepository) ListByVersionIDs(context.Context, ...int64) (*collectionlist.List[packagedomain.ProjectPackageFile], error) {
	return collectionlist.NewList(r.file), nil
}
func (r *conditionalFileRepository) GetByID(context.Context, int64) (packagedomain.ProjectPackageFile, error) {
	return r.file, nil
}
func (r *conditionalFileRepository) Create(context.Context, storageports.CreateProjectPackageFileInput) (packagedomain.ProjectPackageFile, error) {
	return r.file, nil
}
func (r *conditionalFileRepository) MarkStored(context.Context, int64, storageports.StoreProjectPackageFileInput) error {
	return nil
}
func (r *conditionalFileRepository) DeleteByID(context.Context, int64) error { return nil }

type conditionalObjectStorage struct {
	mu        sync.Mutex
	content   []byte
	loadCalls int
}

func (s *conditionalObjectStorage) SaveObject(context.Context, string, []byte, string) error {
	return nil
}
func (s *conditionalObjectStorage) SaveIssueAttachment(context.Context, string, int64, int64, string, []byte, string) (string, error) {
	return "", nil
}
func (s *conditionalObjectStorage) SavePackageFile(context.Context, string, string, string, string, int64, string, []byte, string) (string, error) {
	return "", nil
}
func (s *conditionalObjectStorage) SaveLFSObject(context.Context, string, string, []byte) (string, error) {
	return "", nil
}
func (s *conditionalObjectStorage) SavePipelineArtifact(context.Context, string, int64, int64, string, []byte, string) (string, error) {
	return "", nil
}
func (s *conditionalObjectStorage) Load(context.Context, string) ([]byte, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.loadCalls++
	return append([]byte(nil), s.content...), nil
}
func (s *conditionalObjectStorage) loadCount() int {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.loadCalls
}
