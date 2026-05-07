package projectbranchprotection

import (
	"context"
	"fmt"
	projectports "github.com/DaiYuANg/gity/internal/application/ports"
	projectdomain "github.com/DaiYuANg/gity/internal/domain/project"
	persistence "github.com/DaiYuANg/gity/internal/infrastructure/persistence"
	dbschema "github.com/DaiYuANg/gity/internal/infrastructure/persistence/db_schema"
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[projectdomain.ProjectBranchProtection, dbschema.ProjectBranchProtectionSchemaDef]
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[projectdomain.ProjectBranchProtection](db, dbschema.ProjectBranchProtectionSchema, dbxrepo.WithByIDNotFoundAsError(true)),
	}, nil
}

func NewProjectBranchProtectionRepository(repo *Repository) projectports.ProjectBranchProtectionRepository {
	return repo
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[projectdomain.ProjectBranchProtection], error) {
	query := querydsl.Select(dbschema.ProjectBranchProtectionSchema.AllColumns().Values()...).
		From(dbschema.ProjectBranchProtectionSchema).
		Where(dbschema.ProjectBranchProtectionSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectBranchProtectionSchema.BranchName.Asc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByProjectAndBranch(ctx context.Context, projectID int64, branchName string) (projectdomain.ProjectBranchProtection, error) {
	query := querydsl.Select(dbschema.ProjectBranchProtectionSchema.AllColumns().Values()...).
		From(dbschema.ProjectBranchProtectionSchema).
		Where(querydsl.And(
			dbschema.ProjectBranchProtectionSchema.ProjectID.Eq(projectID),
			dbschema.ProjectBranchProtectionSchema.BranchName.Eq(normalizeBranchName(branchName)),
		)).
		Limit(1)
	return persistence.One(r.base.First(ctx, query))
}

func (r *Repository) Protect(ctx context.Context, projectID int64, branchName string) (projectdomain.ProjectBranchProtection, error) {
	branchName = normalizeBranchName(branchName)
	if existing, err := r.GetByProjectAndBranch(ctx, projectID, branchName); err == nil {
		return existing, nil
	} else if !persistence.IsNotFound(err) {
		return projectdomain.ProjectBranchProtection{}, err
	}
	now := time.Now().UTC()
	item := projectdomain.ProjectBranchProtection{
		ProjectID:  projectID,
		BranchName: branchName,
		CreatedAt:  now,
		UpdatedAt:  now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return projectdomain.ProjectBranchProtection{}, fmt.Errorf("insert project branch protection: %w", err)
	}
	return item, nil
}

func (r *Repository) Unprotect(ctx context.Context, projectID int64, branchName string) error {
	item, err := r.GetByProjectAndBranch(ctx, projectID, branchName)
	if err != nil {
		if persistence.IsNotFound(err) {
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
