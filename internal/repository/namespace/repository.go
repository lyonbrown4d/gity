package namespace

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/DaiYuANg/arcgo/collectionx"
	"github.com/DaiYuANg/arcgo/dbx"
	"github.com/DaiYuANg/gity/internal/entity"
)

type Repository struct {
	db     *dbx.DB
	mapper dbx.StructMapper[entity.Namespace]
}

type CreateInput struct {
	Kind        string
	Name        string
	PathKey     string
	Description string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	mapper, err := dbx.NewStructMapper[entity.Namespace]()
	if err != nil {
		return nil, fmt.Errorf("new namespace mapper: %w", err)
	}
	return &Repository{db: db, mapper: mapper}, nil
}

func (r *Repository) List(ctx context.Context) (collectionx.List[entity.Namespace], error) {
	statement := dbx.NewSQLStatement("namespace.list", func(params any) (dbx.BoundQuery, error) {
		_ = params
		return dbx.BoundQuery{
			SQL: "SELECT id, kind, name, path_key, full_path, description, created_at, updated_at FROM namespaces ORDER BY id DESC",
		}, nil
	})
	return dbx.SQLList(ctx, r.db, statement, nil, r.mapper)
}

func (r *Repository) GetByID(ctx context.Context, id int64) (entity.Namespace, error) {
	statement := dbx.NewSQLStatement("namespace.get_by_id", func(params any) (dbx.BoundQuery, error) {
		value, ok := params.(int64)
		if !ok {
			return dbx.BoundQuery{}, fmt.Errorf("namespace.get_by_id expects int64")
		}
		return dbx.BoundQuery{
			SQL:  "SELECT id, kind, name, path_key, full_path, description, created_at, updated_at FROM namespaces WHERE id = ?",
			Args: collectionx.NewList[any](value),
		}, nil
	})
	return dbx.SQLGet(ctx, r.db, statement, id, r.mapper)
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (entity.Namespace, error) {
	trimmedPath := strings.TrimSpace(input.PathKey)
	trimmedName := strings.TrimSpace(input.Name)
	now := time.Now().UTC()

	result, err := r.db.ExecContext(
		ctx,
		"INSERT INTO namespaces (kind, name, path_key, full_path, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
		strings.TrimSpace(input.Kind),
		trimmedName,
		trimmedPath,
		trimmedPath,
		strings.TrimSpace(input.Description),
		now,
		now,
	)
	if err != nil {
		return entity.Namespace{}, fmt.Errorf("insert namespace: %w", err)
	}
	id, err := result.LastInsertId()
	if err != nil {
		return entity.Namespace{}, fmt.Errorf("read namespace id: %w", err)
	}
	return r.GetByID(ctx, id)
}
