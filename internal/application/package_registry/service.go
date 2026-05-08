package packageregistry

import (
	"context"
	"encoding/base64"
	"errors"
	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	storageports "github.com/DaiYuANg/gity/internal/application/ports"
	packagedomain "github.com/DaiYuANg/gity/internal/domain/package_registry"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	collectionlist "github.com/arcgolabs/collectionx/list"
	mappingx "github.com/arcgolabs/collectionx/mapping"
	"github.com/samber/oops"
	"strings"
)

type Service struct {
	projectRepo storageports.ProjectRepository
	packageRepo storageports.ProjectPackageRepository
	versionRepo storageports.ProjectPackageVersionRepository
	fileRepo    storageports.ProjectPackageFileRepository
	storage     storageports.ObjectStorage
}

type UploadFileInput struct {
	Type          string `json:"type"`
	Name          string `json:"name"`
	Version       string `json:"version"`
	FileName      string `json:"file_name"`
	FilePath      string `json:"file_path"`
	ContentType   string `json:"content_type"`
	ContentBase64 string `json:"content_base64"`
}

type PackageDetail struct {
	Package  packagedomain.ProjectPackage `json:"package"`
	Versions []PackageVersionDetail       `json:"versions"`
}

type PackageVersionDetail struct {
	Version packagedomain.ProjectPackageVersion `json:"version"`
	Files   []packagedomain.ProjectPackageFile  `json:"files"`
}

type PackageFileContent struct {
	File    packagedomain.ProjectPackageFile `json:"file"`
	Content string                           `json:"content_base64"`
}

type normalizedUploadFile struct {
	PackageType string
	PackageName string
	Version     string
	FileName    string
	FilePath    string
	ContentType string
	Content     []byte
}

func NewService(projectRepo storageports.ProjectRepository, packageRepo storageports.ProjectPackageRepository, versionRepo storageports.ProjectPackageVersionRepository, fileRepo storageports.ProjectPackageFileRepository, storage storageports.ObjectStorage) *Service {
	return &Service{projectRepo: projectRepo, packageRepo: packageRepo, versionRepo: versionRepo, fileRepo: fileRepo, storage: storage}
}

func (s *Service) ListPackages(ctx context.Context, projectID int64) ([]packagedomain.ProjectPackage, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	items, err := s.packageRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, oops.In("package_registry").With("project_id", projectID).Wrapf(err, "list project packages")
	}
	return items.Values(), nil
}

func (s *Service) GetPackage(ctx context.Context, projectID, packageID int64) (PackageDetail, error) {
	pkg, err := s.loadPackage(ctx, projectID, packageID)
	if err != nil {
		return PackageDetail{}, err
	}
	versions, err := s.versionRepo.ListByPackageID(ctx, pkg.ID)
	if err != nil {
		return PackageDetail{}, oops.In("package_registry").With("project_id", projectID, "package_id", packageID).Wrapf(err, "list package versions")
	}
	versionIDs := collectionlist.MapList(versions, func(_ int, version packagedomain.ProjectPackageVersion) int64 {
		return version.ID
	}).Values()
	files, err := s.fileRepo.ListByVersionIDs(ctx, versionIDs...)
	if err != nil {
		return PackageDetail{}, oops.In("package_registry").With("project_id", projectID, "package_id", packageID).Wrapf(err, "list package version files")
	}
	filesByVersion := mappingx.GroupByList(files, func(_ int, file packagedomain.ProjectPackageFile) int64 {
		return file.ProjectPackageVersionID
	})
	return PackageDetail{
		Package: pkg,
		Versions: collectionlist.MapList(versions, func(_ int, version packagedomain.ProjectPackageVersion) PackageVersionDetail {
			return PackageVersionDetail{Version: version, Files: filesByVersion.GetCopy(version.ID)}
		}).Values(),
	}, nil
}

func (s *Service) UploadFile(ctx context.Context, projectID int64, input UploadFileInput) (packagedomain.ProjectPackageFile, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return packagedomain.ProjectPackageFile{}, apperror.NotFound("project not found", err)
	}
	normalized, err := normalizeUploadFileInput(projectID, input)
	if err != nil {
		return packagedomain.ProjectPackageFile{}, err
	}
	pkg, err := s.getOrCreatePackage(ctx, projectID, normalized)
	if err != nil {
		return packagedomain.ProjectPackageFile{}, err
	}
	versionRecord, err := s.getOrCreateVersion(ctx, projectID, pkg.ID, normalized)
	if err != nil {
		return packagedomain.ProjectPackageFile{}, err
	}
	return s.createAndStoreFile(ctx, project, pkg, versionRecord, normalized)
}

func normalizeUploadFileInput(projectID int64, input UploadFileInput) (normalizedUploadFile, error) {
	packageType := strings.TrimSpace(input.Type)
	packageName := strings.TrimSpace(input.Name)
	versionValue := strings.TrimSpace(input.Version)
	fileName := strings.TrimSpace(input.FileName)
	filePath := strings.TrimSpace(input.FilePath)
	if packageType == "" || packageName == "" || versionValue == "" || fileName == "" {
		return normalizedUploadFile{}, oops.In("package_registry").With("project_id", projectID, "type", packageType, "name", packageName, "version", versionValue, "file_name", fileName).New("type, name, version, and file_name are required")
	}
	if strings.TrimSpace(input.ContentBase64) == "" {
		return normalizedUploadFile{}, oops.In("package_registry").With("project_id", projectID, "type", packageType, "name", packageName, "version", versionValue, "file_name", fileName).New("content_base64 is required")
	}
	content, err := base64.StdEncoding.DecodeString(strings.TrimSpace(input.ContentBase64))
	if err != nil {
		return normalizedUploadFile{}, oops.In("package_registry").With("project_id", projectID, "type", packageType, "name", packageName, "version", versionValue, "file_name", fileName).Wrapf(err, "decode package file content")
	}
	contentType := strings.TrimSpace(input.ContentType)
	if contentType == "" {
		contentType = storageports.DetectContentType(fileName)
	}
	if filePath == "" {
		filePath = fileName
	}
	return normalizedUploadFile{PackageType: packageType, PackageName: packageName, Version: versionValue, FileName: fileName, FilePath: filePath, ContentType: contentType, Content: content}, nil
}

func (s *Service) getOrCreatePackage(ctx context.Context, projectID int64, input normalizedUploadFile) (packagedomain.ProjectPackage, error) {
	pkg, err := s.packageRepo.GetByProjectTypeAndName(ctx, projectID, input.PackageType, input.PackageName)
	if err != nil {
		if !errors.Is(err, storageports.ErrNotFound) {
			return packagedomain.ProjectPackage{}, oops.In("package_registry").With("project_id", projectID, "type", input.PackageType, "name", input.PackageName).Wrapf(err, "load package")
		}

		pkg, err = s.packageRepo.Create(ctx, storageports.CreateProjectPackageInput{ProjectID: projectID, Type: input.PackageType, Name: input.PackageName})
		if err != nil {
			return packagedomain.ProjectPackage{}, oops.In("package_registry").With("project_id", projectID, "type", input.PackageType, "name", input.PackageName).Wrapf(err, "create package")
		}
	}
	return pkg, nil
}

func (s *Service) getOrCreateVersion(ctx context.Context, projectID, packageID int64, input normalizedUploadFile) (packagedomain.ProjectPackageVersion, error) {
	versionRecord, err := s.versionRepo.GetByPackageAndVersion(ctx, packageID, input.Version)
	if err != nil {
		if !errors.Is(err, storageports.ErrNotFound) {
			return packagedomain.ProjectPackageVersion{}, oops.In("package_registry").With("project_id", projectID, "package_id", packageID, "version", input.Version).Wrapf(err, "load package version")
		}

		versionRecord, err = s.versionRepo.Create(ctx, storageports.CreateProjectPackageVersionInput{ProjectPackageID: packageID, Version: input.Version, Status: "default"})
		if err != nil {
			return packagedomain.ProjectPackageVersion{}, oops.In("package_registry").With("project_id", projectID, "package_id", packageID, "version", input.Version).Wrapf(err, "create package version")
		}
	}
	return versionRecord, nil
}

func (s *Service) createAndStoreFile(ctx context.Context, project projectdomain.Project, pkg packagedomain.ProjectPackage, versionRecord packagedomain.ProjectPackageVersion, input normalizedUploadFile) (packagedomain.ProjectPackageFile, error) {
	fileRecord, err := s.fileRepo.Create(ctx, storageports.CreateProjectPackageFileInput{ProjectPackageVersionID: versionRecord.ID, FileName: input.FileName, FilePath: input.FilePath, ContentType: input.ContentType})
	if err != nil {
		return packagedomain.ProjectPackageFile{}, oops.In("package_registry").With("project_id", project.ID, "package_id", pkg.ID, "version_id", versionRecord.ID, "file_name", input.FileName).Wrapf(err, "create package file record")
	}
	storageKey, err := s.storage.SavePackageFile(ctx, project.FullPath, input.PackageType, input.PackageName, input.Version, fileRecord.ID, input.FileName, input.Content, input.ContentType)
	if err != nil {
		if cleanupErr := s.fileRepo.DeleteByID(ctx, fileRecord.ID); cleanupErr != nil {
			return packagedomain.ProjectPackageFile{}, oops.In("package_registry").With("project_id", project.ID, "file_id", fileRecord.ID, "type", input.PackageType, "name", input.PackageName, "version", input.Version).Wrapf(oops.Join(err, cleanupErr), "save package file and cleanup record")
		}
		return packagedomain.ProjectPackageFile{}, oops.In("package_registry").With("project_id", project.ID, "file_id", fileRecord.ID, "type", input.PackageType, "name", input.PackageName, "version", input.Version).Wrapf(err, "save package file")
	}
	if storeErr := s.fileRepo.MarkStored(ctx, fileRecord.ID, storageports.StoreProjectPackageFileInput{ContentType: input.ContentType, ByteSize: int64(len(input.Content)), StorageKey: storageKey}); storeErr != nil {
		return packagedomain.ProjectPackageFile{}, oops.In("package_registry").With("project_id", project.ID, "file_id", fileRecord.ID, "storage_key", storageKey).Wrapf(storeErr, "mark package file stored")
	}
	storedFile, err := s.fileRepo.GetByID(ctx, fileRecord.ID)
	if err != nil {
		return packagedomain.ProjectPackageFile{}, oops.In("package_registry").With("project_id", project.ID, "file_id", fileRecord.ID).Wrapf(err, "reload package file")
	}
	return storedFile, nil
}

func (s *Service) GetFileContent(ctx context.Context, projectID, fileID int64) (PackageFileContent, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return PackageFileContent{}, apperror.NotFound("project not found", err)
	}
	fileRecord, err := s.fileRepo.GetByID(ctx, fileID)
	if err != nil {
		if errors.Is(err, storageports.ErrNotFound) {
			return PackageFileContent{}, apperror.NotFound("package file not found", err)
		}
		return PackageFileContent{}, oops.In("package_registry").With("project_id", projectID, "file_id", fileID).Wrapf(err, "load package file")
	}
	content, err := s.storage.Load(ctx, fileRecord.StorageKey)
	if err != nil {
		return PackageFileContent{}, apperror.NotFound("package file content not found", err)
	}
	return PackageFileContent{File: fileRecord, Content: base64.StdEncoding.EncodeToString(content)}, nil
}

func (s *Service) loadPackage(ctx context.Context, projectID, packageID int64) (packagedomain.ProjectPackage, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return packagedomain.ProjectPackage{}, apperror.NotFound("project not found", err)
	}
	pkg, err := s.packageRepo.GetByID(ctx, packageID)
	if err != nil {
		if errors.Is(err, storageports.ErrNotFound) {
			return packagedomain.ProjectPackage{}, apperror.NotFound("package not found", err)
		}
		return packagedomain.ProjectPackage{}, oops.In("package_registry").With("project_id", projectID, "package_id", packageID).Wrapf(err, "load package")
	}
	if pkg.ProjectID != projectID {
		return packagedomain.ProjectPackage{}, apperror.NotFound("package not found", nil)
	}
	return pkg, nil
}
