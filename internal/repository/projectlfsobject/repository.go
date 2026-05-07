package projectlfsobject

import (
	"context"
	"fmt"
	"strings"
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
	"github.com/DaiYuANg/gity/internal/entity"
	dbxrepo "github.com/arcgolabs/dbx/repository"
)

type Repository struct {
	base *dbxrepo.Base[entity.ProjectLFSObject, entity.ProjectLFSObjectSchemaDef]
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[entity.ProjectLFSObject](db, entity.ProjectLFSObjectSchema, dbxrepo.WithByIDNotFoundAsError(true))}, nil
}

func (r *Repository) GetByProjectAndOID(ctx context.Context, projectID int64, oid string) (entity.ProjectLFSObject, error) {
	query := dbx.Select(entity.ProjectLFSObjectSchema.AllColumns().Values()...).From(entity.ProjectLFSObjectSchema).Where(dbx.And(entity.ProjectLFSObjectSchema.ProjectID.Eq(projectID), entity.ProjectLFSObjectSchema.OID.Eq(strings.TrimSpace(oid)))).Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, projectID int64, oid string, byteSize int64, storageKey string) (entity.ProjectLFSObject, error) {
	now := time.Now().UTC()
	item := entity.ProjectLFSObject{ProjectID: projectID, OID: strings.TrimSpace(oid), ByteSize: byteSize, StorageKey: strings.TrimSpace(storageKey), CreatedAt: now, UpdatedAt: now}
	if err := r.base.Create(ctx, &item); err != nil {
		return entity.ProjectLFSObject{}, fmt.Errorf("insert project lfs object: %w", err)
	}
	return item, nil
}

func (r *Repository) UpdateStored(ctx context.Context, id int64, byteSize int64, storageKey string) error {
	_, err := r.base.UpdateByID(ctx, id,
		entity.ProjectLFSObjectSchema.ByteSize.Set(byteSize),
		entity.ProjectLFSObjectSchema.StorageKey.Set(strings.TrimSpace(storageKey)),
		entity.ProjectLFSObjectSchema.UpdatedAt.Set(time.Now().UTC()),
	)
	if err != nil {
		return fmt.Errorf("update project lfs object: %w", err)
	}
	return nil
}
