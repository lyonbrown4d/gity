package packageregistry

import (
	"context"
	"encoding/base64"
	"fmt"
	"net/http"
	"strings"

	dbxrepo "github.com/DaiYuANg/arcgo/dbx/repository"
	"github.com/DaiYuANg/arcgo/httpx"
	"github.com/DaiYuANg/gity/internal/entity"
	platformstorage "github.com/DaiYuANg/gity/internal/platform/storage"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	projectpackagerepo "github.com/DaiYuANg/gity/internal/repository/projectpackage"
	projectpackagefilerepo "github.com/DaiYuANg/gity/internal/repository/projectpackagefile"
	projectpackageversionrepo "github.com/DaiYuANg/gity/internal/repository/projectpackageversion"
)

type Service struct {
	projectRepo *projectrepo.Repository
	packageRepo *projectpackagerepo.Repository
	versionRepo *projectpackageversionrepo.Repository
	fileRepo    *projectpackagefilerepo.Repository
	storage     *platformstorage.Service
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
	Package  entity.ProjectPackage  `json:"package"`
	Versions []PackageVersionDetail `json:"versions"`
}

type PackageVersionDetail struct {
	Version entity.ProjectPackageVersion `json:"version"`
	Files   []entity.ProjectPackageFile  `json:"files"`
}

type PackageFileContent struct {
	File    entity.ProjectPackageFile `json:"file"`
	Content string                    `json:"content_base64"`
}

func NewService(projectRepo *projectrepo.Repository, packageRepo *projectpackagerepo.Repository, versionRepo *projectpackageversionrepo.Repository, fileRepo *projectpackagefilerepo.Repository, storage *platformstorage.Service) *Service {
	return &Service{projectRepo: projectRepo, packageRepo: packageRepo, versionRepo: versionRepo, fileRepo: fileRepo, storage: storage}
}

func (s *Service) ListPackages(ctx context.Context, projectID int64) ([]entity.ProjectPackage, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	items, err := s.packageRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, err
	}
	return items.Values(), nil
}

func (s *Service) GetPackage(ctx context.Context, projectID int64, packageID int64) (PackageDetail, error) {
	pkg, err := s.loadPackage(ctx, projectID, packageID)
	if err != nil {
		return PackageDetail{}, err
	}
	versions, err := s.versionRepo.ListByPackageID(ctx, pkg.ID)
	if err != nil {
		return PackageDetail{}, err
	}
	detail := PackageDetail{Package: pkg, Versions: make([]PackageVersionDetail, 0, versions.Len())}
	for _, version := range versions.Values() {
		files, fileErr := s.fileRepo.ListByVersionID(ctx, version.ID)
		if fileErr != nil {
			return PackageDetail{}, fileErr
		}
		detail.Versions = append(detail.Versions, PackageVersionDetail{Version: version, Files: files.Values()})
	}
	return detail, nil
}

func (s *Service) UploadFile(ctx context.Context, projectID int64, input UploadFileInput) (entity.ProjectPackageFile, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return entity.ProjectPackageFile{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	packageType := strings.TrimSpace(input.Type)
	packageName := strings.TrimSpace(input.Name)
	versionValue := strings.TrimSpace(input.Version)
	fileName := strings.TrimSpace(input.FileName)
	filePath := strings.TrimSpace(input.FilePath)
	if packageType == "" || packageName == "" || versionValue == "" || fileName == "" {
		return entity.ProjectPackageFile{}, fmt.Errorf("type, name, version, and file_name are required")
	}
	if strings.TrimSpace(input.ContentBase64) == "" {
		return entity.ProjectPackageFile{}, fmt.Errorf("content_base64 is required")
	}
	content, err := base64.StdEncoding.DecodeString(strings.TrimSpace(input.ContentBase64))
	if err != nil {
		return entity.ProjectPackageFile{}, fmt.Errorf("decode package file content: %w", err)
	}
	pkg, err := s.packageRepo.GetByProjectTypeAndName(ctx, projectID, packageType, packageName)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			pkg, err = s.packageRepo.Create(ctx, projectpackagerepo.CreateInput{ProjectID: projectID, Type: packageType, Name: packageName})
		}
		if err != nil {
			return entity.ProjectPackageFile{}, err
		}
	}
	versionRecord, err := s.versionRepo.GetByPackageAndVersion(ctx, pkg.ID, versionValue)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			versionRecord, err = s.versionRepo.Create(ctx, projectpackageversionrepo.CreateInput{ProjectPackageID: pkg.ID, Version: versionValue, Status: "default"})
		}
		if err != nil {
			return entity.ProjectPackageFile{}, err
		}
	}
	contentType := strings.TrimSpace(input.ContentType)
	if contentType == "" {
		contentType = platformstorage.DetectContentType(fileName)
	}
	storedPath := filePath
	if strings.TrimSpace(storedPath) == "" {
		storedPath = fileName
	}
	fileRecord, err := s.fileRepo.Create(ctx, projectpackagefilerepo.CreateInput{ProjectPackageVersionID: versionRecord.ID, FileName: fileName, FilePath: storedPath, ContentType: contentType})
	if err != nil {
		return entity.ProjectPackageFile{}, err
	}
	storageKey, err := s.storage.SavePackageFile(ctx, project.FullPath, packageType, packageName, versionValue, fileRecord.ID, fileName, content, contentType)
	if err != nil {
		_ = s.fileRepo.DeleteByID(ctx, fileRecord.ID)
		return entity.ProjectPackageFile{}, err
	}
	if err := s.fileRepo.MarkStored(ctx, fileRecord.ID, projectpackagefilerepo.StoreInput{ContentType: contentType, ByteSize: int64(len(content)), StorageKey: storageKey}); err != nil {
		return entity.ProjectPackageFile{}, err
	}
	return s.fileRepo.GetByID(ctx, fileRecord.ID)
}

func (s *Service) GetFileContent(ctx context.Context, projectID int64, fileID int64) (PackageFileContent, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return PackageFileContent{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	fileRecord, err := s.fileRepo.GetByID(ctx, fileID)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return PackageFileContent{}, httpx.NewError(http.StatusNotFound, "package file not found", err)
		}
		return PackageFileContent{}, err
	}
	content, err := s.storage.Load(ctx, fileRecord.StorageKey)
	if err != nil {
		return PackageFileContent{}, httpx.NewError(http.StatusNotFound, "package file content not found", err)
	}
	return PackageFileContent{File: fileRecord, Content: base64.StdEncoding.EncodeToString(content)}, nil
}

func (s *Service) loadPackage(ctx context.Context, projectID int64, packageID int64) (entity.ProjectPackage, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return entity.ProjectPackage{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	pkg, err := s.packageRepo.GetByID(ctx, packageID)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return entity.ProjectPackage{}, httpx.NewError(http.StatusNotFound, "package not found", err)
		}
		return entity.ProjectPackage{}, err
	}
	if pkg.ProjectID != projectID {
		return entity.ProjectPackage{}, httpx.NewError(http.StatusNotFound, "package not found", nil)
	}
	return pkg, nil
}
