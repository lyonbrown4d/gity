package lfs

import (
	"context"
	"fmt"
	storageports "github.com/DaiYuANg/gity/internal/application/ports"
	identity "github.com/DaiYuANg/gity/internal/domain/identity"
	lfsdomain "github.com/DaiYuANg/gity/internal/domain/lfs"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectlfslockrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectlfslock"
	projectlfsobjectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectlfsobject"
	userrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user"
	setx "github.com/arcgolabs/collectionx/set"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"github.com/arcgolabs/httpx"
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
	projectRepo *projectrepo.Repository
	objectRepo  *projectlfsobjectrepo.Repository
	lockRepo    *projectlfslockrepo.Repository
	userRepo    *userrepo.Repository
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

func NewService(projectRepo *projectrepo.Repository, objectRepo *projectlfsobjectrepo.Repository, lockRepo *projectlfslockrepo.Repository, userRepo *userrepo.Repository, storage storageports.ObjectStorage) *Service {
	return &Service{projectRepo: projectRepo, objectRepo: objectRepo, lockRepo: lockRepo, userRepo: userRepo, storage: storage}
}

func (s *Service) PrepareBatch(ctx context.Context, projectID int64, request BatchRequest, baseURL string, repoHTTPPath string) (BatchResponse, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return BatchResponse{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	operation := strings.TrimSpace(strings.ToLower(request.Operation))
	if !supportedBatchOperations.Contains(operation) {
		return BatchResponse{}, fmt.Errorf("unsupported lfs operation: %s", request.Operation)
	}
	response := BatchResponse{Transfer: "basic", Objects: make([]BatchObjectResult, 0, len(request.Objects))}
	for _, object := range request.Objects {
		oid := strings.TrimSpace(object.OID)
		item := BatchObjectResult{OID: oid, Size: object.Size}
		if err := validateOID(oid); err != nil {
			item.Error = &ActionError{Code: http.StatusBadRequest, Message: err.Error()}
			response.Objects = append(response.Objects, item)
			continue
		}
		if object.Size < 0 {
			item.Error = &ActionError{Code: http.StatusBadRequest, Message: "lfs object size must be non-negative"}
			response.Objects = append(response.Objects, item)
			continue
		}
		record, err := s.objectRepo.GetByProjectAndOID(ctx, projectID, oid)
		exists := err == nil
		if err != nil && err != dbxrepo.ErrNotFound {
			return BatchResponse{}, err
		}
		endpoint := strings.TrimRight(baseURL, "/") + "/" + strings.Trim(strings.ReplaceAll(repoHTTPPath, "\\", "/"), "/") + "/info/lfs/objects/" + oid
		switch operation {
		case "upload":
			if !exists {
				item.Actions = map[string]ActionDetail{"upload": {Href: endpoint}}
			}
		case "download":
			if !exists {
				item.Error = &ActionError{Code: http.StatusNotFound, Message: "lfs object not found"}
			} else {
				item.Size = record.ByteSize
				item.Actions = map[string]ActionDetail{"download": {Href: endpoint}}
			}
		}
		response.Objects = append(response.Objects, item)
	}
	return response, nil
}

func (s *Service) UploadObject(ctx context.Context, projectID int64, oid string, content []byte) (lfsdomain.ProjectLFSObject, error) {
	project, err := s.projectRepo.GetByID(ctx, projectID)
	if err != nil {
		return lfsdomain.ProjectLFSObject{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	oid = strings.TrimSpace(oid)
	if err := validateOID(oid); err != nil {
		return lfsdomain.ProjectLFSObject{}, httpx.NewError(http.StatusBadRequest, "invalid lfs oid", err)
	}
	storageKey := storageports.BuildLFSStorageKey(project.FullPath, oid)
	if err := s.storage.SaveObject(ctx, storageKey, content, "application/octet-stream"); err != nil {
		return lfsdomain.ProjectLFSObject{}, err
	}
	record, err := s.objectRepo.GetByProjectAndOID(ctx, projectID, oid)
	if err == nil {
		if err := s.objectRepo.UpdateStored(ctx, record.ID, int64(len(content)), storageKey); err != nil {
			return lfsdomain.ProjectLFSObject{}, err
		}
		return s.objectRepo.GetByProjectAndOID(ctx, projectID, oid)
	}
	if err != nil && err != dbxrepo.ErrNotFound {
		return lfsdomain.ProjectLFSObject{}, err
	}
	return s.objectRepo.Create(ctx, projectID, oid, int64(len(content)), storageKey)
}

func (s *Service) DownloadObject(ctx context.Context, projectID int64, oid string) (DownloadObject, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return DownloadObject{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	oid = strings.TrimSpace(oid)
	if err := validateOID(oid); err != nil {
		return DownloadObject{}, httpx.NewError(http.StatusBadRequest, "invalid lfs oid", err)
	}
	record, err := s.objectRepo.GetByProjectAndOID(ctx, projectID, oid)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return DownloadObject{}, httpx.NewError(http.StatusNotFound, "lfs object not found", err)
		}
		return DownloadObject{}, err
	}
	content, err := s.storage.Load(ctx, record.StorageKey)
	if err != nil {
		return DownloadObject{}, httpx.NewError(http.StatusNotFound, "lfs object content not found", err)
	}
	return DownloadObject{Object: record, Content: content}, nil
}

func (s *Service) CreateLock(ctx context.Context, projectID int64, ownerUserID int64, input CreateLockInput) (LockEnvelope, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return LockEnvelope{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	owner, err := s.userRepo.GetByID(ctx, ownerUserID)
	if err != nil {
		return LockEnvelope{}, httpx.NewError(http.StatusNotFound, "lock owner not found", err)
	}
	lockPath, err := normalizeLockPath(input.Path)
	if err != nil {
		return LockEnvelope{}, httpx.NewError(http.StatusBadRequest, "invalid lock path", err)
	}
	if _, err := s.lockRepo.GetByProjectAndPath(ctx, projectID, lockPath); err == nil {
		return LockEnvelope{}, httpx.NewError(http.StatusConflict, "lfs lock already exists", fmt.Errorf("lfs lock path already exists: %s", lockPath))
	} else if err != nil && err != dbxrepo.ErrNotFound {
		return LockEnvelope{}, err
	}
	lock, err := s.lockRepo.Create(ctx, projectlfslockrepo.CreateInput{ProjectID: projectID, OwnerUserID: ownerUserID, Path: lockPath})
	if err != nil {
		return LockEnvelope{}, err
	}
	return LockEnvelope{Lock: buildLockView(lock, owner)}, nil
}

func (s *Service) ListLocks(ctx context.Context, projectID int64, input LockListInput) (LockListResult, error) {
	items, nextCursor, err := s.listLockEntities(ctx, projectID, input)
	if err != nil {
		return LockListResult{}, err
	}
	result := LockListResult{Locks: make([]LockView, 0, len(items)), NextCursor: nextCursor}
	for _, item := range items {
		view, err := s.buildLockView(ctx, item)
		if err != nil {
			return LockListResult{}, err
		}
		result.Locks = append(result.Locks, view)
	}
	return result, nil
}

func (s *Service) VerifyLocks(ctx context.Context, projectID int64, currentUserID int64, input LockListInput) (VerifyLocksResult, error) {
	items, nextCursor, err := s.listLockEntities(ctx, projectID, input)
	if err != nil {
		return VerifyLocksResult{}, err
	}
	result := VerifyLocksResult{Ours: make([]LockView, 0, len(items)), Theirs: make([]LockView, 0, len(items)), NextCursor: nextCursor}
	for _, item := range items {
		view, err := s.buildLockView(ctx, item)
		if err != nil {
			return VerifyLocksResult{}, err
		}
		if item.OwnerUserID == currentUserID {
			result.Ours = append(result.Ours, view)
		} else {
			result.Theirs = append(result.Theirs, view)
		}
	}
	return result, nil
}

func (s *Service) Unlock(ctx context.Context, projectID int64, actorUserID int64, lockID string, input UnlockInput) (LockEnvelope, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return LockEnvelope{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	parsedID, err := parseLockID(lockID)
	if err != nil {
		return LockEnvelope{}, httpx.NewError(http.StatusBadRequest, "invalid lock id", err)
	}
	item, err := s.lockRepo.GetByProjectAndID(ctx, projectID, parsedID)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return LockEnvelope{}, httpx.NewError(http.StatusNotFound, "lfs lock not found", err)
		}
		return LockEnvelope{}, err
	}
	if item.OwnerUserID != actorUserID && !input.Force {
		return LockEnvelope{}, httpx.NewError(http.StatusForbidden, "lfs lock is owned by another user", fmt.Errorf("lock owner mismatch"))
	}
	owner, err := s.userRepo.GetByID(ctx, item.OwnerUserID)
	if err != nil {
		return LockEnvelope{}, httpx.NewError(http.StatusNotFound, "lock owner not found", err)
	}
	if err := s.lockRepo.DeleteByID(ctx, item.ID); err != nil {
		return LockEnvelope{}, err
	}
	return LockEnvelope{Lock: buildLockView(item, owner)}, nil
}

func (s *Service) listLockEntities(ctx context.Context, projectID int64, input LockListInput) ([]lfsdomain.ProjectLFSLock, string, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, "", httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	if strings.TrimSpace(input.ID) != "" {
		lockID, err := parseLockID(input.ID)
		if err != nil {
			return nil, "", httpx.NewError(http.StatusBadRequest, "invalid lock id", err)
		}
		item, err := s.lockRepo.GetByProjectAndID(ctx, projectID, lockID)
		if err != nil {
			if err == dbxrepo.ErrNotFound {
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
			return nil, "", httpx.NewError(http.StatusBadRequest, "invalid lock path", err)
		}
	}
	afterID, err := parseCursor(input.Cursor)
	if err != nil {
		return nil, "", httpx.NewError(http.StatusBadRequest, "invalid lock cursor", err)
	}
	limit := normalizeLockLimit(input.Limit)
	items, err := s.lockRepo.ListByProjectID(ctx, projectlfslockrepo.ListInput{ProjectID: projectID, Path: lockPath, AfterID: afterID, Limit: limit + 1})
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
		return LockView{}, httpx.NewError(http.StatusNotFound, "lock owner not found", err)
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
		return "", fmt.Errorf("lfs lock path is required")
	}
	for _, segment := range strings.Split(trimmed, "/") {
		if segment == "" || segment == "." || segment == ".." {
			return "", fmt.Errorf("lfs lock path is invalid")
		}
	}
	cleaned := strings.Trim(path.Clean("/"+trimmed), "/")
	if cleaned == "" || strings.HasPrefix(cleaned, "../") || cleaned == ".." {
		return "", fmt.Errorf("lfs lock path is invalid")
	}
	return cleaned, nil
}

func validateOID(oid string) error {
	if len(oid) != 64 {
		return fmt.Errorf("lfs oid must be a 64-character hex string")
	}
	for _, r := range oid {
		if !unicode.Is(unicode.ASCII_Hex_Digit, r) {
			return fmt.Errorf("lfs oid must be a 64-character hex string")
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
		return 0, fmt.Errorf("invalid cursor")
	}
	return parsed, nil
}

func parseLockID(value string) (int64, error) {
	trimmed := strings.TrimSpace(value)
	if trimmed == "" {
		return 0, fmt.Errorf("invalid lock id")
	}
	parsed, err := strconv.ParseInt(trimmed, 10, 64)
	if err != nil || parsed <= 0 {
		return 0, fmt.Errorf("invalid lock id")
	}
	return parsed, nil
}

func normalizeLockLimit(limit int) int {
	if limit <= 0 || limit > defaultLockPageSize {
		return defaultLockPageSize
	}
	return limit
}
