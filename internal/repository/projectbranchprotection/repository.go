package projectbranchprotection

import (
	"context"
	"fmt"
	"strings"
	"time"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
	"github.com/DaiYuANg/gity/internal/entity"
	collectionx "github.com/arcgolabs/collectionx/list"
	dbxrepo "github.com/arcgolabs/dbx/repository"
)

type Repository struct {
	base *dbxrepo.Base[entity.ProjectBranchProtection, entity.ProjectBranchProtectionSchemaDef]
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[entity.ProjectBranchProtection](db, entity.ProjectBranchProtectionSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[entity.ProjectBranchProtection], error) {
	query := dbx.Select(entity.ProjectBranchProtectionSchema.AllColumns().Values()...).
		From(entity.ProjectBranchProtectionSchema).
		Where(entity.ProjectBranchProtectionSchema.ProjectID.Eq(projectID)).
		OrderBy(entity.ProjectBranchProtectionSchema.BranchName.Asc())
	return r.base.List(ctx, query)
}

func (r *Repository) GetByProjectAndBranch(ctx context.Context, projectID int64, branchName string) (entity.ProjectBranchProtection, error) {
	query := dbx.Select(entity.ProjectBranchProtectionSchema.AllColumns().Values()...).
		From(entity.ProjectBranchProtectionSchema).
		Where(dbx.And(
			entity.ProjectBranchProtectionSchema.ProjectID.Eq(projectID),
			entity.ProjectBranchProtectionSchema.BranchName.Eq(normalizeBranchName(branchName)),
		)).
		Limit(1)
	return r.base.First(ctx, query)
}

func (r *Repository) Protect(ctx context.Context, projectID int64, branchName string) (entity.ProjectBranchProtection, error) {
	branchName = normalizeBranchName(branchName)
	if existing, err := r.GetByProjectAndBranch(ctx, projectID, branchName); err == nil {
		return existing, nil
	} else if err != dbxrepo.ErrNotFound {
		return entity.ProjectBranchProtection{}, err
	}
	now := time.Now().UTC()
	item := entity.ProjectBranchProtection{
		ProjectID:  projectID,
		BranchName: branchName,
		CreatedAt:  now,
		UpdatedAt:  now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return entity.ProjectBranchProtection{}, fmt.Errorf("insert project branch protection: %w", err)
	}
	return item, nil
}

func (r *Repository) Unprotect(ctx context.Context, projectID int64, branchName string) error {
	item, err := r.GetByProjectAndBranch(ctx, projectID, branchName)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return nil
		}
		return err
	}
	if _, err := r.base.DeleteByID(ctx, item.ID); err != nil {
		return fmt.Errorf("delete project branch protection: %w", err)
	}
	return nil
}

func normalizeBranchName(value string) string {
	return strings.TrimSpace(value)
}
