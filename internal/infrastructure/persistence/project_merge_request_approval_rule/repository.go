package projectmergerequestapprovalrule

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	mergeports "github.com/lyonbrown4d/gity/internal/application/ports"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
	persistence "github.com/lyonbrown4d/gity/internal/infrastructure/persistence"
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
	"github.com/samber/oops"
)

type Repository struct {
	base *dbxrepo.Base[mergedomain.ProjectMergeRequestApprovalRule, dbschema.ProjectMergeRequestApprovalRuleSchemaDef]
}

type CreateInput = mergeports.CreateProjectMergeRequestApprovalRuleInput
type UpdateInput = mergeports.UpdateProjectMergeRequestApprovalRuleInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[mergedomain.ProjectMergeRequestApprovalRule](db, dbschema.ProjectMergeRequestApprovalRuleSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectMergeRequestApprovalRuleRepository(repo *Repository) mergeports.ProjectMergeRequestApprovalRuleRepository {
	return repo
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[mergedomain.ProjectMergeRequestApprovalRule], error) {
	query := querydsl.Select(dbschema.ProjectMergeRequestApprovalRuleSchema.AllColumns().Values()...).
		From(dbschema.ProjectMergeRequestApprovalRuleSchema).
		Where(dbschema.ProjectMergeRequestApprovalRuleSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectMergeRequestApprovalRuleSchema.ID.Desc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) GetByProjectAndID(ctx context.Context, projectID, id int64) (mergedomain.ProjectMergeRequestApprovalRule, error) {
	query := querydsl.Select(dbschema.ProjectMergeRequestApprovalRuleSchema.AllColumns().Values()...).
		From(dbschema.ProjectMergeRequestApprovalRuleSchema).
		Where(querydsl.And(
			dbschema.ProjectMergeRequestApprovalRuleSchema.ProjectID.Eq(projectID),
			dbschema.ProjectMergeRequestApprovalRuleSchema.ID.Eq(id),
		)).
		Limit(1)
	item, err := r.base.First(ctx, query)
	if err != nil {
		if persistence.IsNotFound(err) {
			return mergedomain.ProjectMergeRequestApprovalRule{}, mergeports.ErrNotFound
		}
		return mergedomain.ProjectMergeRequestApprovalRule{}, oops.In("persistence.merge_request_approval_rule").With("project_id", projectID, "rule_id", id).Wrapf(err, "load merge request approval rule")
	}
	return item, nil
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (mergedomain.ProjectMergeRequestApprovalRule, error) {
	now := time.Now().UTC()
	item := mergedomain.ProjectMergeRequestApprovalRule{
		ProjectID:         input.ProjectID,
		Name:              strings.TrimSpace(input.Name),
		TargetBranch:      strings.TrimSpace(input.TargetBranch),
		ApprovalsRequired: input.ApprovalsRequired,
		EligibleUserIDs:   strings.TrimSpace(input.EligibleUserIDs),
		CodeOwner:         input.CodeOwner,
		CreatedAt:         now,
		UpdatedAt:         now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return mergedomain.ProjectMergeRequestApprovalRule{}, oops.In("persistence.merge_request_approval_rule").With("project_id", input.ProjectID, "name", input.Name).Wrapf(err, "insert merge request approval rule")
	}
	return item, nil
}

func (r *Repository) UpdateByID(ctx context.Context, id int64, input UpdateInput) error {
	patch := dbxrepo.PatchSet(r.base, approvalRuleKey(id)).
		Set(dbschema.ProjectMergeRequestApprovalRuleSchema.UpdatedAt.Set(time.Now().UTC()))
	if input.Name != nil {
		patch = patch.Set(dbschema.ProjectMergeRequestApprovalRuleSchema.Name.Set(strings.TrimSpace(*input.Name)))
	}
	if input.TargetBranch != nil {
		patch = patch.Set(dbschema.ProjectMergeRequestApprovalRuleSchema.TargetBranch.Set(strings.TrimSpace(*input.TargetBranch)))
	}
	if input.ApprovalsRequired != nil {
		patch = patch.Set(dbschema.ProjectMergeRequestApprovalRuleSchema.ApprovalsRequired.Set(*input.ApprovalsRequired))
	}
	if input.EligibleUserIDs != nil {
		patch = patch.Set(dbschema.ProjectMergeRequestApprovalRuleSchema.EligibleUserIDs.Set(strings.TrimSpace(*input.EligibleUserIDs)))
	}
	if input.CodeOwner != nil {
		patch = patch.Set(dbschema.ProjectMergeRequestApprovalRuleSchema.CodeOwner.Set(*input.CodeOwner))
	}
	if _, err := patch.Apply(ctx); err != nil {
		return fmt.Errorf("update merge request approval rule: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByProjectAndID(ctx context.Context, projectID, id int64) error {
	item, err := r.GetByProjectAndID(ctx, projectID, id)
	if err != nil {
		if errors.Is(err, mergeports.ErrNotFound) {
			return nil
		}
		return err
	}
	if _, err := r.base.DeleteByKeySet(ctx, approvalRuleKey(item.ID)); err != nil {
		return fmt.Errorf("delete merge request approval rule: %w", err)
	}
	return nil
}

func approvalRuleKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectMergeRequestApprovalRuleSchema.ID, id))
}
