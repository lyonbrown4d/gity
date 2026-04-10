package project

import (
	"context"
	"database/sql"
	"fmt"
	"strings"
	"time"

	"github.com/DaiYuANg/arcgo/collectionx"
	"github.com/DaiYuANg/arcgo/dbx"
	"github.com/DaiYuANg/gity/internal/entity"
)

type Repository struct {
	db     *dbx.DB
	mapper dbx.StructMapper[entity.Project]
}

type CreateInput struct {
	NamespaceID   int64
	Name          string
	PathKey       string
	Description   string
	DefaultBranch string
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	mapper, err := dbx.NewStructMapper[entity.Project]()
	if err != nil {
		return nil, fmt.Errorf("new project mapper: %w", err)
	}
	return &Repository{db: db, mapper: mapper}, nil
}

func (r *Repository) List(ctx context.Context, namespaceID sql.NullInt64) (collectionx.List[entity.Project], error) {
	statement := dbx.NewSQLStatement("project.list", func(params any) (dbx.BoundQuery, error) {
		filter, ok := params.(sql.NullInt64)
		if !ok {
			return dbx.BoundQuery{}, fmt.Errorf("project.list expects sql.NullInt64")
		}
		if filter.Valid {
			return dbx.BoundQuery{
				SQL:  "SELECT id, namespace_id, name, path_key, full_path, description, default_branch, created_at, updated_at FROM projects WHERE namespace_id = ? ORDER BY id DESC",
				Args: collectionx.NewList[any](filter.Int64),
			}, nil
		}
		return dbx.BoundQuery{
			SQL: "SELECT id, namespace_id, name, path_key, full_path, description, default_branch, created_at, updated_at FROM projects ORDER BY id DESC",
		}, nil
	})
	return dbx.SQLList(ctx, r.db, statement, namespaceID, r.mapper)
}

func (r *Repository) GetByID(ctx context.Context, id int64) (entity.Project, error) {
	statement := dbx.NewSQLStatement("project.get_by_id", func(params any) (dbx.BoundQuery, error) {
		value, ok := params.(int64)
		if !ok {
			return dbx.BoundQuery{}, fmt.Errorf("project.get_by_id expects int64")
		}
		return dbx.BoundQuery{
			SQL:  "SELECT id, namespace_id, name, path_key, full_path, description, default_branch, created_at, updated_at FROM projects WHERE id = ?",
			Args: collectionx.NewList[any](value),
		}, nil
	})
	return dbx.SQLGet(ctx, r.db, statement, id, r.mapper)
}

func (r *Repository) Create(ctx context.Context, input CreateInput, namespace entity.Namespace) (entity.Project, error) {
	trimmedPath := strings.TrimSpace(input.PathKey)
	trimmedName := strings.TrimSpace(input.Name)
	defaultBranch := strings.TrimSpace(input.DefaultBranch)
	if defaultBranch == "" {
		defaultBranch = "main"
	}
	fullPath := namespace.FullPath + "/" + trimmedPath
	now := time.Now().UTC()

	result, err := r.db.ExecContext(
		ctx,
		"INSERT INTO projects (namespace_id, name, path_key, full_path, description, default_branch, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
		input.NamespaceID,
		trimmedName,
		trimmedPath,
		fullPath,
		strings.TrimSpace(input.Description),
		defaultBranch,
		now,
		now,
	)
	if err != nil {
		return entity.Project{}, fmt.Errorf("insert project: %w", err)
	}
	id, err := result.LastInsertId()
	if err != nil {
		return entity.Project{}, fmt.Errorf("read project id: %w", err)
	}
	return r.GetByID(ctx, id)
}

func (r *Repository) DeleteByID(ctx context.Context, id int64) error {
	if _, err := r.db.ExecContext(ctx, "DELETE FROM projects WHERE id = ?", id); err != nil {
		return fmt.Errorf("delete project: %w", err)
	}
	return nil
}
