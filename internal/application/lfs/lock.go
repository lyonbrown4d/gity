package lfs

import (
	"context"
	"errors"
	"fmt"
	"path"
	"strconv"
	"strings"

	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	storageports "github.com/DaiYuANg/gity/internal/application/ports"
	identity "github.com/DaiYuANg/gity/internal/domain/identity"
	lfsdomain "github.com/DaiYuANg/gity/internal/domain/lfs"
	collectionlist "github.com/arcgolabs/collectionx/list"
	"github.com/samber/oops"
)

const (
	defaultLockPageSize     = 100
	timeLayoutRFC3339Millis = "2006-01-02T15:04:05.000Z07:00"
)

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
		return LockListResult{}, oops.In("lfs").With("project_id", projectID).Wrapf(err, "build lfs lock list")
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
		return VerifyLocksResult{}, oops.In("lfs").With("project_id", projectID, "current_user_id", currentUserID).Wrapf(err, "build lfs lock verification")
	}
	return result, nil
}

func (s *Service) CreateLock(ctx context.Context, projectID, ownerUserID int64, input CreateLockInput) (LockEnvelope, error) {
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
		return LockEnvelope{}, oops.In("lfs").With("project_id", projectID, "path", lockPath).Wrapf(existingErr, "check existing lfs lock")
	}
	lock, err := s.lockRepo.Create(ctx, storageports.CreateProjectLFSLockInput{ProjectID: projectID, OwnerUserID: ownerUserID, Path: lockPath})
	if err != nil {
		return LockEnvelope{}, oops.In("lfs").With("project_id", projectID, "owner_user_id", ownerUserID, "path", lockPath).Wrapf(err, "create lfs lock")
	}
	return LockEnvelope{Lock: buildLockView(lock, owner)}, nil
}

func (s *Service) Unlock(ctx context.Context, projectID, actorUserID int64, lockID string, input UnlockInput) (LockEnvelope, error) {
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
		return LockEnvelope{}, oops.In("lfs").With("project_id", projectID, "lock_id", parsedID).Wrapf(err, "load lfs lock")
	}
	if item.OwnerUserID != actorUserID && !input.Force {
		return LockEnvelope{}, apperror.Forbidden("lfs lock is owned by another user", errors.New("lock owner mismatch"))
	}
	owner, err := s.userRepo.GetByID(ctx, item.OwnerUserID)
	if err != nil {
		return LockEnvelope{}, apperror.NotFound("lock owner not found", err)
	}
	if err := s.lockRepo.DeleteByID(ctx, item.ID); err != nil {
		return LockEnvelope{}, oops.In("lfs").With("project_id", projectID, "lock_id", item.ID).Wrapf(err, "delete lfs lock")
	}
	return LockEnvelope{Lock: buildLockView(item, owner)}, nil
}

func (s *Service) listLockEntities(ctx context.Context, projectID int64, input LockListInput) ([]lfsdomain.ProjectLFSLock, string, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, "", apperror.NotFound("project not found", err)
	}
	if strings.TrimSpace(input.ID) != "" {
		return s.listLockEntityByID(ctx, projectID, input.ID)
	}
	return s.listLockEntitiesPage(ctx, projectID, input)
}

func (s *Service) listLockEntityByID(ctx context.Context, projectID int64, rawID string) ([]lfsdomain.ProjectLFSLock, string, error) {
	lockID, err := parseLockID(rawID)
	if err != nil {
		return nil, "", apperror.BadRequest("invalid lock id", err)
	}
	item, err := s.lockRepo.GetByProjectAndID(ctx, projectID, lockID)
	if err != nil {
		if errors.Is(err, storageports.ErrNotFound) {
			return []lfsdomain.ProjectLFSLock{}, "", nil
		}
		return nil, "", oops.In("lfs").With("project_id", projectID, "lock_id", lockID).Wrapf(err, "load lfs lock by id")
	}
	return []lfsdomain.ProjectLFSLock{item}, "", nil
}

func (s *Service) listLockEntitiesPage(ctx context.Context, projectID int64, input LockListInput) ([]lfsdomain.ProjectLFSLock, string, error) {
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
		return nil, "", oops.In("lfs").With("project_id", projectID, "path", lockPath, "after_id", afterID, "limit", limit).Wrapf(err, "list lfs locks")
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
