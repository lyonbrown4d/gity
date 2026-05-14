package projectlfsobject

import (
	"context"
	"fmt"
	"strings"
	"time"

	collectionlist "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	lfsports "github.com/lyonbrown4d/gity/internal/application/ports"
	lfsdomain "github.com/lyonbrown4d/gity/internal/domain/lfs"
	persistence "github.com/lyonbrown4d/gity/internal/infrastructure/persistence"
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
)

type Repository struct {
	base *dbxrepo.Base[lfsdomain.ProjectLFSObject, dbschema.ProjectLFSObjectSchemaDef]
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{base: dbxrepo.NewWithOptions[lfsdomain.ProjectLFSObject](db, dbschema.ProjectLFSObjectSchema, dbxrepo.WithKeyNotFoundAsError(true))}, nil
}

func NewProjectLFSObjectRepository(repo *Repository) lfsports.ProjectLFSObjectRepository {
	return repo
}

func (r *Repository) GetByProjectAndOID(ctx context.Context, projectID int64, oid string) (lfsdomain.ProjectLFSObject, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectLFSObjectSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectLFSObjectSchema.OID.Eq(strings.TrimSpace(oid))).
		First(ctx))
}

func (r *Repository) ListByProjectID(ctx context.Context, input lfsports.ListProjectLFSObjectsInput) (*collectionlist.List[lfsdomain.ProjectLFSObject], error) {
	limit := input.Limit
	if limit <= 0 {
		limit = 100
	}
	predicates := []querydsl.Predicate{dbschema.ProjectLFSObjectSchema.ProjectID.Eq(input.ProjectID)}
	if input.AfterID > 0 {
		predicates = append(predicates, dbschema.ProjectLFSObjectSchema.ID.Gt(input.AfterID))
	}
	return persistence.Many(dbxrepo.Query(r.base).
		Where(querydsl.And(predicates...)).
		OrderBy(dbschema.ProjectLFSObjectSchema.ID.Asc()).
		Limit(limit).
		List(ctx))
}

func (r *Repository) Create(ctx context.Context, projectID int64, oid string, byteSize int64, storageKey string) (lfsdomain.ProjectLFSObject, error) {
	now := time.Now().UTC()
	item := lfsdomain.ProjectLFSObject{ProjectID: projectID, OID: strings.TrimSpace(oid), ByteSize: byteSize, StorageKey: strings.TrimSpace(storageKey), CreatedAt: now, UpdatedAt: now}
	if err := r.base.Create(ctx, &item); err != nil {
		return lfsdomain.ProjectLFSObject{}, fmt.Errorf("insert project lfs object: %w", err)
	}
	return item, nil
}

func (r *Repository) UpdateStored(ctx context.Context, id, byteSize int64, storageKey string) error {
	_, err := dbxrepo.PatchSet(r.base, projectLFSObjectKey(id)).Set(
		dbschema.ProjectLFSObjectSchema.ByteSize.Set(byteSize),
		dbschema.ProjectLFSObjectSchema.StorageKey.Set(strings.TrimSpace(storageKey)),
		dbschema.ProjectLFSObjectSchema.UpdatedAt.Set(time.Now().UTC()),
	).Apply(ctx)
	if err != nil {
		return fmt.Errorf("update project lfs object: %w", err)
	}
	return nil
}

func projectLFSObjectKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectLFSObjectSchema.ID, id))
}
