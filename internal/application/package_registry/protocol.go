package packageregistry

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"path"
	"strings"
	"time"

	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	storageports "github.com/lyonbrown4d/gity/internal/application/ports"
	packagedomain "github.com/lyonbrown4d/gity/internal/domain/package_registry"
	"github.com/samber/oops"
)

type UploadRawFileInput struct {
	Type        string
	Name        string
	Version     string
	FileName    string
	FilePath    string
	ContentType string
	Content     []byte
}

type PackageFileBlob struct {
	File         packagedomain.ProjectPackageFile
	Content      []byte
	ETag         string
	LastModified time.Time
}

type PackageFileMetadata struct {
	File         packagedomain.ProjectPackageFile
	ETag         string
	LastModified time.Time
}

func (s *Service) UploadRawFile(ctx context.Context, projectID int64, input UploadRawFileInput) (packagedomain.ProjectPackageFile, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return packagedomain.ProjectPackageFile{}, apperror.NotFound("project not found", err)
	}
	normalized, err := normalizeRawUploadFileInput(projectID, input)
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

func normalizeRawUploadFileInput(projectID int64, input UploadRawFileInput) (normalizedUploadFile, error) {
	packageType := strings.TrimSpace(input.Type)
	packageName := strings.TrimSpace(input.Name)
	versionValue := strings.TrimSpace(input.Version)
	fileName := normalizeFileName(input.FileName)
	filePath := normalizeFilePath(input.FilePath)
	if packageType == "" || packageName == "" || versionValue == "" || fileName == "" {
		return normalizedUploadFile{}, apperror.BadRequest("type, name, version, and file_name are required", oops.In("package_registry").With("project_id", projectID, "type", packageType, "name", packageName, "version", versionValue, "file_name", fileName).New("type, name, version, and file_name are required"))
	}
	if len(input.Content) == 0 {
		return normalizedUploadFile{}, apperror.BadRequest("package file content is required", oops.In("package_registry").With("project_id", projectID, "type", packageType, "name", packageName, "version", versionValue, "file_name", fileName).New("package file content is required"))
	}
	contentType := strings.TrimSpace(input.ContentType)
	if contentType == "" {
		contentType = storageports.DetectContentType(fileName, input.Content)
	}
	if filePath == "" {
		filePath = fileName
	}
	return normalizedUploadFile{PackageType: packageType, PackageName: packageName, Version: versionValue, FileName: fileName, FilePath: filePath, ContentType: contentType, Content: input.Content}, nil
}

func normalizeFileName(value string) string {
	normalized := normalizeFilePath(value)
	if normalized == "" {
		return ""
	}
	return path.Base(normalized)
}

func normalizeFilePath(value string) string {
	return strings.Trim(strings.ReplaceAll(strings.TrimSpace(value), "\\", "/"), "/")
}

func (s *Service) GetFileBlob(ctx context.Context, projectID, fileID int64) (PackageFileBlob, error) {
	metadata, err := s.GetFileMetadata(ctx, projectID, fileID)
	if err != nil {
		return PackageFileBlob{}, err
	}
	content, err := s.storage.Load(ctx, metadata.File.StorageKey)
	if err != nil {
		return PackageFileBlob{}, apperror.NotFound("package file content not found", err)
	}
	return PackageFileBlob{File: metadata.File, Content: content, ETag: metadata.ETag, LastModified: metadata.LastModified}, nil
}

func (s *Service) GetFileMetadata(ctx context.Context, projectID, fileID int64) (PackageFileMetadata, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return PackageFileMetadata{}, apperror.NotFound("project not found", err)
	}
	fileRecord, err := s.fileRepo.GetByID(ctx, fileID)
	if err != nil {
		if errors.Is(err, storageports.ErrNotFound) {
			return PackageFileMetadata{}, apperror.NotFound("package file not found", err)
		}
		return PackageFileMetadata{}, oops.In("package_registry").With("project_id", projectID, "file_id", fileID).Wrapf(err, "load package file")
	}
	if ensureErr := s.ensureFileProject(ctx, projectID, fileRecord); ensureErr != nil {
		return PackageFileMetadata{}, ensureErr
	}
	return packageFileMetadata(fileRecord), nil
}

func (s *Service) GetPackageByTypeAndName(ctx context.Context, projectID int64, packageType, name string) (PackageDetail, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return PackageDetail{}, apperror.NotFound("project not found", err)
	}
	pkg, err := s.packageRepo.GetByProjectTypeAndName(ctx, projectID, strings.TrimSpace(packageType), strings.TrimSpace(name))
	if err != nil {
		if errors.Is(err, storageports.ErrNotFound) {
			return PackageDetail{}, apperror.NotFound("package not found", err)
		}
		return PackageDetail{}, oops.In("package_registry").With("project_id", projectID, "type", packageType, "name", name).Wrapf(err, "load package")
	}
	return s.GetPackage(ctx, projectID, pkg.ID)
}

func (s *Service) GetFileByCoordinate(ctx context.Context, projectID int64, packageType, name, version, filePath string) (PackageFileBlob, error) {
	metadata, err := s.GetFileMetadataByCoordinate(ctx, projectID, packageType, name, version, filePath)
	if err != nil {
		return PackageFileBlob{}, err
	}
	content, err := s.storage.Load(ctx, metadata.File.StorageKey)
	if err != nil {
		return PackageFileBlob{}, apperror.NotFound("package file content not found", err)
	}
	return PackageFileBlob{File: metadata.File, Content: content, ETag: metadata.ETag, LastModified: metadata.LastModified}, nil
}

func (s *Service) GetFileMetadataByCoordinate(ctx context.Context, projectID int64, packageType, name, version, filePath string) (PackageFileMetadata, error) {
	detail, err := s.GetPackageByTypeAndName(ctx, projectID, packageType, name)
	if err != nil {
		return PackageFileMetadata{}, err
	}
	fileRecord, ok := findPackageFileByCoordinate(detail, version, filePath)
	if !ok {
		return PackageFileMetadata{}, apperror.NotFound("package file not found", storageports.ErrNotFound)
	}
	return packageFileMetadata(fileRecord), nil
}

func packageFileMetadata(file packagedomain.ProjectPackageFile) PackageFileMetadata {
	fingerprint := fmt.Sprintf(
		"%d:%d:%q:%q:%q:%d:%q:%d",
		file.ID,
		file.ProjectPackageVersionID,
		file.FileName,
		file.FilePath,
		file.ContentType,
		file.ByteSize,
		file.StorageKey,
		file.UpdatedAt.UTC().UnixNano(),
	)
	sum := sha256.Sum256([]byte(fingerprint))
	return PackageFileMetadata{
		File:         file,
		ETag:         hex.EncodeToString(sum[:]),
		LastModified: file.UpdatedAt.UTC().Truncate(time.Second),
	}
}

func findPackageFileByCoordinate(detail PackageDetail, version, filePath string) (packagedomain.ProjectPackageFile, bool) {
	targetVersion := strings.TrimSpace(version)
	targetPath := normalizeFilePath(filePath)
	targetName := path.Base(targetPath)
	for index := range detail.Versions {
		versionDetail := detail.Versions[index]
		if versionDetail.Version.Version != targetVersion {
			continue
		}
		if fileRecord, ok := findPackageFileByPath(versionDetail.Files, targetPath, targetName); ok {
			return fileRecord, true
		}
	}
	return packagedomain.ProjectPackageFile{}, false
}

func findPackageFileByPath(files []packagedomain.ProjectPackageFile, targetPath, targetName string) (packagedomain.ProjectPackageFile, bool) {
	for index := range files {
		fileRecord := files[index]
		if normalizeFilePath(fileRecord.FilePath) == targetPath || fileRecord.FileName == targetName {
			return fileRecord, true
		}
	}
	return packagedomain.ProjectPackageFile{}, false
}

func (s *Service) ensureFileProject(ctx context.Context, projectID int64, fileRecord packagedomain.ProjectPackageFile) error {
	version, err := s.versionRepo.GetByID(ctx, fileRecord.ProjectPackageVersionID)
	if err != nil {
		return oops.In("package_registry").With("project_id", projectID, "file_id", fileRecord.ID, "version_id", fileRecord.ProjectPackageVersionID).Wrapf(err, "load package file version")
	}
	pkg, err := s.packageRepo.GetByID(ctx, version.ProjectPackageID)
	if err != nil {
		return oops.In("package_registry").With("project_id", projectID, "file_id", fileRecord.ID, "package_id", version.ProjectPackageID).Wrapf(err, "load package file package")
	}
	if pkg.ProjectID != projectID {
		return apperror.NotFound("package file not found", nil)
	}
	return nil
}
