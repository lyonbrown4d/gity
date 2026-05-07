package ports

import (
	"context"

	lfsdomain "github.com/DaiYuANg/gity/internal/domain/lfs"
	collectionx "github.com/arcgolabs/collectionx/list"
)

type ProjectLFSObjectRepository interface {
	GetByProjectAndOID(ctx context.Context, projectID int64, oid string) (lfsdomain.ProjectLFSObject, error)
	Create(ctx context.Context, projectID int64, oid string, byteSize int64, storageKey string) (lfsdomain.ProjectLFSObject, error)
	UpdateStored(ctx context.Context, id int64, byteSize int64, storageKey string) error
}

type ProjectLFSLockRepository interface {
	GetByProjectAndID(ctx context.Context, projectID int64, id int64) (lfsdomain.ProjectLFSLock, error)
	GetByProjectAndPath(ctx context.Context, projectID int64, path string) (lfsdomain.ProjectLFSLock, error)
	ListByProjectID(ctx context.Context, input ListProjectLFSLocksInput) (*collectionx.List[lfsdomain.ProjectLFSLock], error)
	Create(ctx context.Context, input CreateProjectLFSLockInput) (lfsdomain.ProjectLFSLock, error)
	DeleteByID(ctx context.Context, id int64) error
}

type CreateProjectLFSLockInput struct {
	ProjectID   int64
	OwnerUserID int64
	Path        string
}

type ListProjectLFSLocksInput struct {
	ProjectID int64
	Path      string
	AfterID   int64
	Limit     int
}
