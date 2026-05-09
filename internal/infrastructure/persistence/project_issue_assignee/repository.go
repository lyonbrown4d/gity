package projectissueassignee

import (
	"context"
	"time"

	issueports "github.com/DaiYuANg/gity/internal/application/ports"
	issuedomain "github.com/DaiYuANg/gity/internal/domain/issue"
	persistence "github.com/DaiYuANg/gity/internal/infrastructure/persistence"
	dbschema "github.com/DaiYuANg/gity/internal/infrastructure/persistence/db_schema"
	collectionx "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"github.com/samber/oops"
)

type Repository struct {
	base *dbxrepo.Base[issuedomain.ProjectIssueAssignee, dbschema.ProjectIssueAssigneeSchemaDef]
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[issuedomain.ProjectIssueAssignee](db, dbschema.ProjectIssueAssigneeSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectIssueAssigneeRepository(repo *Repository) issueports.ProjectIssueAssigneeRepository {
	return repo
}

func (r *Repository) ListByIssueID(ctx context.Context, issueID int64) (*collectionx.List[issuedomain.ProjectIssueAssignee], error) {
	query := querydsl.Select(dbschema.ProjectIssueAssigneeSchema.AllColumns().Values()...).
		From(dbschema.ProjectIssueAssigneeSchema).
		Where(dbschema.ProjectIssueAssigneeSchema.ProjectIssueID.Eq(issueID)).
		OrderBy(dbschema.ProjectIssueAssigneeSchema.ID.Asc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) ReplaceByIssueID(ctx context.Context, issueID int64, userIDs []int64) (*collectionx.List[issuedomain.ProjectIssueAssignee], error) {
	orderedUserIDs := uniquePositiveInt64s(userIDs)
	var assignees *collectionx.List[issuedomain.ProjectIssueAssignee]
	err := r.base.InTx(ctx, nil, func(_ *dbx.Tx, repo *dbxrepo.Base[issuedomain.ProjectIssueAssignee, dbschema.ProjectIssueAssigneeSchemaDef]) error {
		listed, err := replaceIssueAssignees(ctx, repo, issueID, orderedUserIDs)
		if err != nil {
			return err
		}
		assignees = listed
		return nil
	})
	if err != nil {
		return nil, oops.In("persistence.project_issue_assignee").With("issue_id", issueID).Wrapf(err, "replace issue assignees")
	}
	return persistence.Many(assignees, nil)
}

func replaceIssueAssignees(ctx context.Context, repo *dbxrepo.Base[issuedomain.ProjectIssueAssignee, dbschema.ProjectIssueAssigneeSchemaDef], issueID int64, userIDs []int64) (*collectionx.List[issuedomain.ProjectIssueAssignee], error) {
	if err := deleteIssueAssignees(ctx, repo, issueID); err != nil {
		return nil, err
	}
	if err := insertIssueAssignees(ctx, repo, issueID, userIDs); err != nil {
		return nil, err
	}
	return listIssueAssignees(ctx, repo, issueID)
}

func deleteIssueAssignees(ctx context.Context, repo *dbxrepo.Base[issuedomain.ProjectIssueAssignee, dbschema.ProjectIssueAssigneeSchemaDef], issueID int64) error {
	deleteQuery := querydsl.DeleteFrom(dbschema.ProjectIssueAssigneeSchema).
		Where(dbschema.ProjectIssueAssigneeSchema.ProjectIssueID.Eq(issueID))
	if _, err := repo.Delete(ctx, deleteQuery); err != nil {
		return oops.In("persistence.project_issue_assignee").With("issue_id", issueID).Wrapf(err, "delete issue assignees")
	}
	return nil
}

func insertIssueAssignees(ctx context.Context, repo *dbxrepo.Base[issuedomain.ProjectIssueAssignee, dbschema.ProjectIssueAssigneeSchemaDef], issueID int64, userIDs []int64) error {
	if len(userIDs) == 0 {
		return nil
	}
	now := time.Now().UTC()
	items := make([]*issuedomain.ProjectIssueAssignee, 0, len(userIDs))
	for _, userID := range userIDs {
		items = append(items, &issuedomain.ProjectIssueAssignee{
			ProjectIssueID: issueID,
			UserID:         userID,
			CreatedAt:      now,
			UpdatedAt:      now,
		})
	}
	if err := repo.CreateMany(ctx, items...); err != nil {
		return oops.In("persistence.project_issue_assignee").With("issue_id", issueID).Wrapf(err, "insert issue assignees")
	}
	return nil
}

func listIssueAssignees(ctx context.Context, repo *dbxrepo.Base[issuedomain.ProjectIssueAssignee, dbschema.ProjectIssueAssigneeSchemaDef], issueID int64) (*collectionx.List[issuedomain.ProjectIssueAssignee], error) {
	query := querydsl.Select(dbschema.ProjectIssueAssigneeSchema.AllColumns().Values()...).
		From(dbschema.ProjectIssueAssigneeSchema).
		Where(dbschema.ProjectIssueAssigneeSchema.ProjectIssueID.Eq(issueID)).
		OrderBy(dbschema.ProjectIssueAssigneeSchema.ID.Asc())
	listed, err := repo.List(ctx, query)
	if err != nil {
		return nil, oops.In("persistence.project_issue_assignee").With("issue_id", issueID).Wrapf(err, "list issue assignees")
	}
	return listed, nil
}

func uniquePositiveInt64s(values []int64) []int64 {
	seen := setx.NewOrderedSetWithCapacity[int64](len(values))
	for _, value := range values {
		if value > 0 {
			seen.Add(value)
		}
	}
	return seen.Values()
}
