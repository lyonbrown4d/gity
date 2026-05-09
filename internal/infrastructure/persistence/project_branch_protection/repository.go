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
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"strings"
	"time"
)

type Repository struct {
	base *dbxrepo.Base[projectdomain.ProjectBranchProtection, dbschema.ProjectBranchProtectionSchemaDef]
}

type UpsertInput = projectports.UpsertProjectBranchProtectionInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[projectdomain.ProjectBranchProtection](db, dbschema.ProjectBranchProtectionSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectBranchProtectionRepository(repo *Repository) projectports.ProjectBranchProtectionRepository {
	return repo
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[projectdomain.ProjectBranchProtection], error) {
	return persistence.Many(dbxrepo.Query(r.base).
		Where(dbschema.ProjectBranchProtectionSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectBranchProtectionSchema.BranchName.Asc()).
		List(ctx))
}

func (r *Repository) GetByProjectAndBranch(ctx context.Context, projectID int64, branchName string) (projectdomain.ProjectBranchProtection, error) {
	return persistence.One(dbxrepo.Query(r.base).
		Where(dbschema.ProjectBranchProtectionSchema.ProjectID.Eq(projectID)).
		Where(dbschema.ProjectBranchProtectionSchema.BranchName.Eq(normalizeBranchName(branchName))).
		First(ctx))
}

func (r *Repository) MatchByProjectAndBranch(ctx context.Context, projectID int64, branchName string) (projectdomain.ProjectBranchProtection, error) {
	branchName = normalizeBranchName(branchName)
	items, err := r.ListByProjectID(ctx, projectID)
	if err != nil {
		return projectdomain.ProjectBranchProtection{}, err
	}
	var patternMatch projectdomain.ProjectBranchProtection
	patternMatched := false
	values := items.Values()
	for index := range values {
		item := &values[index]
		if !item.MatchesBranch(branchName) {
			continue
		}
		if projectdomain.NormalizeProjectBranchProtectionRuleType(item.RuleType, item.BranchName) == projectdomain.ProjectBranchProtectionRuleExact {
			return *item, nil
		}
		if !patternMatched {
			patternMatch = *item
			patternMatched = true
		}
	}
	if patternMatched {
		return patternMatch, nil
	}
	return projectdomain.ProjectBranchProtection{}, projectports.ErrNotFound
}

func (r *Repository) Protect(ctx context.Context, projectID int64, branchName string) (projectdomain.ProjectBranchProtection, error) {
	branchName = normalizeBranchName(branchName)
	if existing, err := r.GetByProjectAndBranch(ctx, projectID, branchName); err == nil {
		return existing, nil
	} else if !persistence.IsNotFound(err) {
		return projectdomain.ProjectBranchProtection{}, err
	}
	now := time.Now().UTC()
	item := projectdomain.NewProjectBranchProtection(projectID, branchName, now)
	if err := r.base.Create(ctx, &item); err != nil {
		return projectdomain.ProjectBranchProtection{}, fmt.Errorf("insert project branch protection: %w", err)
	}
	return item, nil
}

func (r *Repository) Upsert(ctx context.Context, projectID int64, input UpsertInput) (projectdomain.ProjectBranchProtection, error) {
	branchName := normalizeBranchName(input.BranchName)
	now := time.Now().UTC()
	item := projectdomain.NewProjectBranchProtection(projectID, branchName, now)
	item.RuleType = projectdomain.NormalizeProjectBranchProtectionRuleType(input.RuleType, branchName)
	item.PushAccessLevel = projectdomain.NormalizeProjectBranchProtectionAccessLevel(input.PushAccessLevel, projectdomain.ProjectBranchProtectionAccessNoOne)
	item.MergeAccessLevel = projectdomain.NormalizeProjectBranchProtectionAccessLevel(input.MergeAccessLevel, projectdomain.ProjectBranchProtectionAccessMaintainer)
	item.RequireMergeRequest = boolInt(input.RequireMergeRequest)
	item.RequirePipelineSuccess = boolInt(input.RequirePipelineSuccess)
	item.AllowForcePush = boolInt(input.AllowForcePush)
	item.AllowDelete = boolInt(input.AllowDelete)

	existing, err := r.GetByProjectAndBranch(ctx, projectID, branchName)
	if err != nil {
		if !persistence.IsNotFound(err) {
			return projectdomain.ProjectBranchProtection{}, err
		}
		if err := r.base.Create(ctx, &item); err != nil {
			return projectdomain.ProjectBranchProtection{}, fmt.Errorf("insert project branch protection: %w", err)
		}
		return item, nil
	}
	if _, err := dbxrepo.PatchSet(r.base, projectBranchProtectionKey(existing.ID)).Set(
		dbschema.ProjectBranchProtectionSchema.RuleType.Set(item.RuleType),
		dbschema.ProjectBranchProtectionSchema.PushAccessLevel.Set(item.PushAccessLevel),
		dbschema.ProjectBranchProtectionSchema.MergeAccessLevel.Set(item.MergeAccessLevel),
		dbschema.ProjectBranchProtectionSchema.RequireMergeRequest.Set(item.RequireMergeRequest),
		dbschema.ProjectBranchProtectionSchema.RequirePipelineSuccess.Set(item.RequirePipelineSuccess),
		dbschema.ProjectBranchProtectionSchema.AllowForcePush.Set(item.AllowForcePush),
		dbschema.ProjectBranchProtectionSchema.AllowDelete.Set(item.AllowDelete),
		dbschema.ProjectBranchProtectionSchema.UpdatedAt.Set(now),
	).Apply(ctx); err != nil {
		return projectdomain.ProjectBranchProtection{}, fmt.Errorf("update project branch protection: %w", err)
	}
	return r.GetByProjectAndBranch(ctx, projectID, branchName)
}

func (r *Repository) Unprotect(ctx context.Context, projectID int64, branchName string) error {
	item, err := r.GetByProjectAndBranch(ctx, projectID, branchName)
	if err != nil {
		if persistence.IsNotFound(err) {
			return nil
		}
		return err
	}
	if _, err := r.base.DeleteByKeySet(ctx, projectBranchProtectionKey(item.ID)); err != nil {
		return fmt.Errorf("delete project branch protection: %w", err)
	}
	return nil
}

func projectBranchProtectionKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectBranchProtectionSchema.ID, id))
}

func normalizeBranchName(value string) string {
	return strings.TrimSpace(value)
}

func boolInt(value bool) int {
	if value {
		return 1
	}
	return 0
}
