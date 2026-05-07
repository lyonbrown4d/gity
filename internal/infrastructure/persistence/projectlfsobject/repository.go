package projectlfsobject

import (
	"context"
	"fmt"
	lfsdomain "github.com/DaiYuANg/gity/internal/domain/lfs"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[lfsdomain.ProjectLFSObject, lfsdomain.ProjectLFSObjectSchemaDef]
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[lfsdomain.ProjectLFSObject](db, lfsdomain.ProjectLFSObjectSchema, dbxrepo.WithByIDNotFoundAsError(true))}, nil
}

func (r *Repository) GetByProjectAndOID(ctx context.Context, projectID int64, oid string) (lfsdomain.ProjectLFSObject, error) {
	query := querydsl.Select(lfsdomain.ProjectLFSObjectSchema.AllColumns().Values()...).From(lfsdomain.ProjectLFSObjectSchema).Where(querydsl.And(lfsdomain.ProjectLFSObjectSchema.ProjectID.Eq(projectID), lfsdomain.ProjectLFSObjectSchema.OID.Eq(strings.TrimSpace(oid)))).Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Create(ctx context.Context, projectID int64, oid string, byteSize int64, storageKey string) (lfsdomain.ProjectLFSObject, error) {
	now := time.Now().UTC()
	item := lfsdomain.ProjectLFSObject{ProjectID: projectID, OID: strings.TrimSpace(oid), ByteSize: byteSize, StorageKey: strings.TrimSpace(storageKey), CreatedAt: now, UpdatedAt: now}
	if err := r.base.Create(ctx, &item); err != nil {
		return lfsdomain.ProjectLFSObject{}, fmt.Errorf("insert project lfs object: %w", err)
	}
	return item, nil
}

func (r *Repository) UpdateStored(ctx context.Context, id int64, byteSize int64, storageKey string) error {
	_, err := r.base.UpdateByID(ctx, id,
		lfsdomain.ProjectLFSObjectSchema.ByteSize.Set(byteSize),
		lfsdomain.ProjectLFSObjectSchema.StorageKey.Set(strings.TrimSpace(storageKey)),
		lfsdomain.ProjectLFSObjectSchema.UpdatedAt.Set(time.Now().UTC()),
	)
	if err != nil {
		return fmt.Errorf("update project lfs object: %w", err)
	}
	return nil
}
