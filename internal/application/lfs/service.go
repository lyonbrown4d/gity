package lfs

import (
	"context"
	"errors"
	"net/http"
	"strings"
	"unicode"

	collectionlist "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	storageports "github.com/lyonbrown4d/gity/internal/application/ports"
	lfsdomain "github.com/lyonbrown4d/gity/internal/domain/lfs"
	"github.com/samber/oops"
)

var supportedBatchOperations = setx.NewSet("upload", "download")

type Service struct {
	projectRepo storageports.ProjectRepository
	objectRepo  storageports.ProjectLFSObjectRepository
	lockRepo    storageports.ProjectLFSLockRepository
	userRepo    storageports.UserRepository
	storage     storageports.ObjectStorage
}

type BatchRequest struct {
	Operation string               `json:"operation"`
	Transfers []string             `json:"transfers"`
	Objects   []BatchObjectRequest `json:"objects"`
}

type BatchObjectRequest struct {
	OID  string `json:"oid"`
	Size int64  `json:"size"`
}

type BatchResponse struct {
	Transfer string              `json:"transfer"`
	Objects  []BatchObjectResult `json:"objects"`
}

type BatchObjectResult struct {
	OID     string                  `json:"oid"`
	Size    int64                   `json:"size"`
	Actions map[string]ActionDetail `json:"actions,omitempty"`
	Error   *ActionError            `json:"error,omitempty"`
}

type ActionDetail struct {
	Href   string            `json:"href"`
	Header map[string]string `json:"header,omitempty"`
}

type ActionError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
}

type DownloadObject struct {
	Object  lfsdomain.ProjectLFSObject
	Content []byte
}

type Dependencies struct {
	ProjectRepo storageports.ProjectRepository
	ObjectRepo  storageports.ProjectLFSObjectRepository
	LockRepo    storageports.ProjectLFSLockRepository
	UserRepo    storageports.UserRepository
	Storage     storageports.ObjectStorage
}

func NewDependencies(projectRepo storageports.ProjectRepository, objectRepo storageports.ProjectLFSObjectRepository, lockRepo storageports.ProjectLFSLockRepository, userRepo storageports.UserRepository, storage storageports.ObjectStorage) Dependencies {
	return Dependencies{ProjectRepo: projectRepo, ObjectRepo: objectRepo, LockRepo: lockRepo, UserRepo: userRepo, Storage: storage}
}

func NewServiceWithDependencies(dependencies Dependencies) *Service {
	return &Service{projectRepo: dependencies.ProjectRepo, objectRepo: dependencies.ObjectRepo, lockRepo: dependencies.LockRepo, userRepo: dependencies.UserRepo, storage: dependencies.Storage}
}

func NewService(projectRepo storageports.ProjectRepository, objectRepo storageports.ProjectLFSObjectRepository, lockRepo storageports.ProjectLFSLockRepository, userRepo storageports.UserRepository, storage storageports.ObjectStorage) *Service {
	return NewServiceWithDependencies(NewDependencies(projectRepo, objectRepo, lockRepo, userRepo, storage))
}

func (s *Service) PrepareBatch(ctx context.Context, projectID int64, request BatchRequest, baseURL, repoHTTPPath string) (BatchResponse, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return BatchResponse{}, apperror.NotFound("project not found", err)
	}
	operation := strings.TrimSpace(strings.ToLower(request.Operation))
	if !supportedBatchOperations.Contains(operation) {
		return BatchResponse{}, oops.In("lfs").With("project_id", projectID, "operation", request.Operation).New("unsupported lfs operation")
	}
	response, err := collectionlist.ReduceErrList(
		collectionlist.NewList(request.Objects...),
		BatchResponse{Transfer: "basic", Objects: make([]BatchObjectResult, 0, len(request.Objects))},
		func(response BatchResponse, _ int, object BatchObjectRequest) (BatchResponse, error) {
			item, err := s.prepareBatchObject(ctx, projectID, operation, object, baseURL, repoHTTPPath)
			if err != nil {
				return BatchResponse{}, err
			}
			response.Objects = append(response.Objects, item)
			return response, nil
		},
	)
	if err != nil {
		return BatchResponse{}, oops.In("lfs").With("project_id", projectID, "operation", operation).Wrapf(err, "prepare lfs batch objects")
	}
	return response, nil
}

func (s *Service) prepareBatchObject(ctx context.Context, projectID int64, operation string, object BatchObjectRequest, baseURL, repoHTTPPath string) (BatchObjectResult, error) {
	oid := strings.TrimSpace(object.OID)
	item := BatchObjectResult{OID: oid, Size: object.Size}
	oidErrorMessage := ""
	if oidErr := validateOID(oid); oidErr != nil {
		oidErrorMessage = oidErr.Error()
	}
	if oidErrorMessage != "" {
		item.Error = &ActionError{Code: http.StatusBadRequest, Message: oidErrorMessage}
		return item, nil
	}
	if object.Size < 0 {
		item.Error = &ActionError{Code: http.StatusBadRequest, Message: "lfs object size must be non-negative"}
		return item, nil
	}
	record, err := s.objectRepo.GetByProjectAndOID(ctx, projectID, oid)
	exists := err == nil
	if err != nil && !errors.Is(err, storageports.ErrNotFound) {
		return BatchObjectResult{}, oops.In("lfs").With("project_id", projectID, "oid", oid).Wrapf(err, "load lfs object")
	}
	endpoint := strings.TrimRight(baseURL, "/") + "/" + strings.Trim(strings.ReplaceAll(repoHTTPPath, "\\", "/"), "/") + "/info/lfs/objects/" + oid
	if operation == "upload" {
		return uploadBatchObjectResult(item, exists, endpoint), nil
	}
	return downloadBatchObjectResult(item, exists, record.ByteSize, endpoint), nil
}

func uploadBatchObjectResult(item BatchObjectResult, exists bool, endpoint string) BatchObjectResult {
	if !exists {
		item.Actions = map[string]ActionDetail{"upload": {Href: endpoint}}
	}
	return item
}

func downloadBatchObjectResult(item BatchObjectResult, exists bool, byteSize int64, endpoint string) BatchObjectResult {
	if !exists {
		item.Error = &ActionError{Code: http.StatusNotFound, Message: "lfs object not found"}
		return item
	}
	item.Size = byteSize
	item.Actions = map[string]ActionDetail{"download": {Href: endpoint}}
	return item
}

func (s *Service) UploadObject(ctx context.Context, projectID int64, oid string, content []byte) (lfsdomain.ProjectLFSObject, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return lfsdomain.ProjectLFSObject{}, apperror.NotFound("project not found", err)
	}
	oid = strings.TrimSpace(oid)
	if oidErr := validateOID(oid); oidErr != nil {
		return lfsdomain.ProjectLFSObject{}, apperror.BadRequest("invalid lfs oid", oidErr)
	}
	storageKey := storageports.BuildLFSStorageKey(project.FullPath, oid)
	if saveErr := s.storage.SaveObject(ctx, storageKey, content, "application/octet-stream"); saveErr != nil {
		return lfsdomain.ProjectLFSObject{}, oops.In("lfs").With("project_id", projectID, "oid", oid, "storage_key", storageKey).Wrapf(saveErr, "save lfs object")
	}
	record, err := s.objectRepo.GetByProjectAndOID(ctx, projectID, oid)
	if err == nil {
		if updateErr := s.objectRepo.UpdateStored(ctx, record.ID, int64(len(content)), storageKey); updateErr != nil {
			return lfsdomain.ProjectLFSObject{}, oops.In("lfs").With("project_id", projectID, "oid", oid, "object_id", record.ID, "storage_key", storageKey).Wrapf(updateErr, "update stored lfs object")
		}
		updated, loadErr := s.objectRepo.GetByProjectAndOID(ctx, projectID, oid)
		if loadErr != nil {
			return lfsdomain.ProjectLFSObject{}, oops.In("lfs").With("project_id", projectID, "oid", oid).Wrapf(loadErr, "load updated lfs object")
		}
		return updated, nil
	}
	if !errors.Is(err, storageports.ErrNotFound) {
		return lfsdomain.ProjectLFSObject{}, oops.In("lfs").With("project_id", projectID, "oid", oid).Wrapf(err, "load lfs object before upload")
	}
	created, err := s.objectRepo.Create(ctx, projectID, oid, int64(len(content)), storageKey)
	if err != nil {
		return lfsdomain.ProjectLFSObject{}, oops.In("lfs").With("project_id", projectID, "oid", oid, "storage_key", storageKey).Wrapf(err, "create lfs object")
	}
	return created, nil
}

func (s *Service) DownloadObject(ctx context.Context, projectID int64, oid string) (DownloadObject, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return DownloadObject{}, apperror.NotFound("project not found", err)
	}
	oid = strings.TrimSpace(oid)
	if err := validateOID(oid); err != nil {
		return DownloadObject{}, apperror.BadRequest("invalid lfs oid", err)
	}
	record, err := s.objectRepo.GetByProjectAndOID(ctx, projectID, oid)
	if err != nil {
		if errors.Is(err, storageports.ErrNotFound) {
			return DownloadObject{}, apperror.NotFound("lfs object not found", err)
		}
		return DownloadObject{}, oops.In("lfs").With("project_id", projectID, "oid", oid).Wrapf(err, "load lfs object")
	}
	content, err := s.storage.Load(ctx, record.StorageKey)
	if err != nil {
		return DownloadObject{}, apperror.NotFound("lfs object content not found", err)
	}
	return DownloadObject{Object: record, Content: content}, nil
}

func validateOID(oid string) error {
	if len(oid) != 64 {
		return errors.New("lfs oid must be a 64-character hex string")
	}
	for _, r := range oid {
		if !unicode.Is(unicode.ASCII_Hex_Digit, r) {
			return errors.New("lfs oid must be a 64-character hex string")
		}
	}
	return nil
}
