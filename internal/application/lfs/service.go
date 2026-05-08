package lfs

import (
	"context"
	"errors"
	"fmt"
	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	storageports "github.com/DaiYuANg/gity/internal/application/ports"
	identity "github.com/DaiYuANg/gity/internal/domain/identity"
	lfsdomain "github.com/DaiYuANg/gity/internal/domain/lfs"
	collectionlist "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	"net/http"
	"path"
	"strconv"
	"strings"
	"unicode"
)

const (
	defaultLockPageSize     = 100
	timeLayoutRFC3339Millis = "2006-01-02T15:04:05.000Z07:00"
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

type CreateLockInput struct {
	Path string `json:"path"`
}

type UnlockInput struct {
	Force bool `json:"force"`
}

type LockListInput struct {
	Path   string
	ID     string
	Cursor string
	Limit  int
}

type LockOwner struct {
	Name string `json:"name"`
}

type LockView struct {
	ID       string    `json:"id"`
	Path     string    `json:"path"`
	LockedAt string    `json:"locked_at"`
	Owner    LockOwner `json:"owner"`
}

type LockEnvelope struct {
	Lock    LockView `json:"lock"`
	Message string   `json:"message,omitempty"`
}

type LockListResult struct {
	Locks      []LockView `json:"locks"`
	NextCursor string     `json:"next_cursor,omitempty"`
}

type VerifyLocksResult struct {
	Ours       []LockView `json:"ours"`
	Theirs     []LockView `json:"theirs"`
	NextCursor string     `json:"next_cursor,omitempty"`
}

func NewService(projectRepo storageports.ProjectRepository, objectRepo storageports.ProjectLFSObjectRepository, lockRepo storageports.ProjectLFSLockRepository, userRepo storageports.UserRepository, storage storageports.ObjectStorage) *Service {
	return &Service{projectRepo: projectRepo, objectRepo: objectRepo, lockRepo: lockRepo, userRepo: userRepo, storage: storage}
}

func (s *Service) PrepareBatch(ctx context.Context, projectID int64, request BatchRequest, baseURL string, repoHTTPPath string) (BatchResponse, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return BatchResponse{}, apperror.NotFound("project not found", err)
	}
	operation := strings.TrimSpace(strings.ToLower(request.Operation))
	if !supportedBatchOperations.Contains(operation) {
		return BatchResponse{}, fmt.Errorf("unsupported lfs operation: %s", request.Operation)
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
		return BatchResponse{}, fmt.Errorf("prepare lfs batch objects: %w", err)
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
		return BatchObjectResult{}, fmt.Errorf("load lfs object: %w", err)
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

func (s *Service) ListLocks(ctx context.Context, projectID int64, input LockListInput) (LockListResult, error) {
	items, nextCursor, err := s.listLockEntities(ctx, projectID, input)
	if err != nil {
		return LockListResult{}, err
	}
	result, err := collectionlist.ReduceErrList(
		collectionlist.NewList(items...),
		LockListResult{Locks: make([]LockView, 0, len(items)), NextCursor: nextCursor},
		func(result LockListResult, _ int, item lfsdomain.ProjectLFSLock) (LockListResult, error) {
			view, viewErr := s.buildLockView(ctx, item)
			if viewErr != nil {
				return LockListResult{}, viewErr
			}
			result.Locks = append(result.Locks, view)
			return result, nil
		},
	)
	if err != nil {
		return LockListResult{}, fmt.Errorf("build lfs lock list: %w", err)
	}
	return result, nil
}

func (s *Service) VerifyLocks(ctx context.Context, projectID, currentUserID int64, input LockListInput) (VerifyLocksResult, error) {
	items, nextCursor, err := s.listLockEntities(ctx, projectID, input)
	if err != nil {
		return VerifyLocksResult{}, err
	}
	result, err := collectionlist.ReduceErrList(
		collectionlist.NewList(items...),
		VerifyLocksResult{Ours: make([]LockView, 0, len(items)), Theirs: make([]LockView, 0, len(items)), NextCursor: nextCursor},
		func(result VerifyLocksResult, _ int, item lfsdomain.ProjectLFSLock) (VerifyLocksResult, error) {
			view, viewErr := s.buildLockView(ctx, item)
			if viewErr != nil {
				return VerifyLocksResult{}, viewErr
			}
			if item.OwnerUserID == currentUserID {
				result.Ours = append(result.Ours, view)
			} else {
				result.Theirs = append(result.Theirs, view)
			}
			return result, nil
		},
	)
	if err != nil {
		return VerifyLocksResult{}, fmt.Errorf("build lfs lock verification: %w", err)
	}
	return result, nil
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
		return lfsdomain.ProjectLFSObject{}, saveErr
	}
	record, err := s.objectRepo.GetByProjectAndOID(ctx, projectID, oid)
	if err == nil {
		if updateErr := s.objectRepo.UpdateStored(ctx, record.ID, int64(len(content)), storageKey); updateErr != nil {
			return lfsdomain.ProjectLFSObject{}, updateErr
		}
		return s.objectRepo.GetByProjectAndOID(ctx, projectID, oid)
	}
	if !errors.Is(err, storageports.ErrNotFound) {
		return lfsdomain.ProjectLFSObject{}, err
	}
	return s.objectRepo.Create(ctx, projectID, oid, int64(len(content)), storageKey)
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
		return DownloadObject{}, err
	}
	content, err := s.storage.Load(ctx, record.StorageKey)
	if err != nil {
		return DownloadObject{}, apperror.NotFound("lfs object content not found", err)
	}
	return DownloadObject{Object: record, Content: content}, nil
}

func (s *Service) CreateLock(ctx context.Context, projectID int64, ownerUserID int64, input CreateLockInput) (LockEnvelope, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return LockEnvelope{}, apperror.NotFound("project not found", err)
	}
	owner, err := s.userRepo.GetByID(ctx, ownerUserID)
	if err != nil {
		return LockEnvelope{}, apperror.NotFound("lock owner not found", err)
	}
	lockPath, err := normalizeLockPath(input.Path)
	if err != nil {
		return LockEnvelope{}, apperror.BadRequest("invalid lock path", err)
	}
	if _, existingErr := s.lockRepo.GetByProjectAndPath(ctx, projectID, lockPath); existingErr == nil {
		return LockEnvelope{}, apperror.Conflict("lfs lock already exists", fmt.Errorf("lfs lock path already exists: %s", lockPath))
	} else if !errors.Is(existingErr, storageports.ErrNotFound) {
		return LockEnvelope{}, existingErr
	}
	lock, err := s.lockRepo.Create(ctx, storageports.CreateProjectLFSLockInput{ProjectID: projectID, OwnerUserID: ownerUserID, Path: lockPath})
	if err != nil {
		return LockEnvelope{}, err
	}
	return LockEnvelope{Lock: buildLockView(lock, owner)}, nil
}

func (s *Service) Unlock(ctx context.Context, projectID int64, actorUserID int64, lockID string, input UnlockInput) (LockEnvelope, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return LockEnvelope{}, apperror.NotFound("project not found", err)
	}
	parsedID, err := parseLockID(lockID)
	if err != nil {
		return LockEnvelope{}, apperror.BadRequest("invalid lock id", err)
	}
	item, err := s.lockRepo.GetByProjectAndID(ctx, projectID, parsedID)
	if err != nil {
		if errors.Is(err, storageports.ErrNotFound) {
			return LockEnvelope{}, apperror.NotFound("lfs lock not found", err)
		}
		return LockEnvelope{}, err
	}
	if item.OwnerUserID != actorUserID && !input.Force {
		return LockEnvelope{}, apperror.Forbidden("lfs lock is owned by another user", errors.New("lock owner mismatch"))
	}
	owner, err := s.userRepo.GetByID(ctx, item.OwnerUserID)
	if err != nil {
		return LockEnvelope{}, apperror.NotFound("lock owner not found", err)
	}
	if err := s.lockRepo.DeleteByID(ctx, item.ID); err != nil {
		return LockEnvelope{}, err
	}
	return LockEnvelope{Lock: buildLockView(item, owner)}, nil
}

func (s *Service) listLockEntities(ctx context.Context, projectID int64, input LockListInput) ([]lfsdomain.ProjectLFSLock, string, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, "", apperror.NotFound("project not found", err)
	}
	if strings.TrimSpace(input.ID) != "" {
		lockID, err := parseLockID(input.ID)
		if err != nil {
			return nil, "", apperror.BadRequest("invalid lock id", err)
		}
		item, err := s.lockRepo.GetByProjectAndID(ctx, projectID, lockID)
		if err != nil {
			if errors.Is(err, storageports.ErrNotFound) {
				return []lfsdomain.ProjectLFSLock{}, "", nil
			}
			return nil, "", err
		}
		return []lfsdomain.ProjectLFSLock{item}, "", nil
	}

	lockPath := ""
	if strings.TrimSpace(input.Path) != "" {
		var err error
		lockPath, err = normalizeLockPath(input.Path)
		if err != nil {
			return nil, "", apperror.BadRequest("invalid lock path", err)
		}
	}
	afterID, err := parseCursor(input.Cursor)
	if err != nil {
		return nil, "", apperror.BadRequest("invalid lock cursor", err)
	}
	limit := normalizeLockLimit(input.Limit)
	items, err := s.lockRepo.ListByProjectID(ctx, storageports.ListProjectLFSLocksInput{ProjectID: projectID, Path: lockPath, AfterID: afterID, Limit: limit + 1})
	if err != nil {
		return nil, "", err
	}
	values := items.Values()
	if len(values) > limit {
		nextCursor := strconv.FormatInt(values[limit-1].ID, 10)
		return values[:limit], nextCursor, nil
	}
	return values, "", nil
}

func (s *Service) buildLockView(ctx context.Context, item lfsdomain.ProjectLFSLock) (LockView, error) {
	owner, err := s.userRepo.GetByID(ctx, item.OwnerUserID)
	if err != nil {
		return LockView{}, apperror.NotFound("lock owner not found", err)
	}
	return buildLockView(item, owner), nil
}

func buildLockView(item lfsdomain.ProjectLFSLock, owner identity.User) LockView {
	name := strings.TrimSpace(owner.DisplayName)
	if name == "" {
		name = strings.TrimSpace(owner.Username)
	}
	return LockView{ID: strconv.FormatInt(item.ID, 10), Path: item.Path, LockedAt: item.CreatedAt.UTC().Format(timeLayoutRFC3339Millis), Owner: LockOwner{Name: name}}
}

func normalizeLockPath(value string) (string, error) {
	trimmed := strings.TrimSpace(strings.ReplaceAll(value, "\\", "/"))
	trimmed = strings.Trim(trimmed, "/")
	if trimmed == "" {
		return "", errors.New("lfs lock path is required")
	}
	for segment := range strings.SplitSeq(trimmed, "/") {
		if segment == "" || segment == "." || segment == ".." {
			return "", errors.New("lfs lock path is invalid")
		}
	}
	cleaned := strings.Trim(path.Clean("/"+trimmed), "/")
	if cleaned == "" || strings.HasPrefix(cleaned, "../") || cleaned == ".." {
		return "", errors.New("lfs lock path is invalid")
	}
	return cleaned, nil
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

func parseCursor(value string) (int64, error) {
	trimmed := strings.TrimSpace(value)
	if trimmed == "" {
		return 0, nil
	}
	parsed, err := strconv.ParseInt(trimmed, 10, 64)
	if err != nil || parsed < 0 {
		return 0, errors.New("invalid cursor")
	}
	return parsed, nil
}

func parseLockID(value string) (int64, error) {
	trimmed := strings.TrimSpace(value)
	if trimmed == "" {
		return 0, errors.New("invalid lock id")
	}
	parsed, err := strconv.ParseInt(trimmed, 10, 64)
	if err != nil || parsed <= 0 {
		return 0, errors.New("invalid lock id")
	}
	return parsed, nil
}

func normalizeLockLimit(limit int) int {
	if limit <= 0 || limit > defaultLockPageSize {
		return defaultLockPageSize
	}
	return limit
}
