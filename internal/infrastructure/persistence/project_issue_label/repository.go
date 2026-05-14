package projectissuelabel

import (
	"context"
	"strings"
	"time"

	collectionx "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	issueports "github.com/lyonbrown4d/gity/internal/application/ports"
	issuedomain "github.com/lyonbrown4d/gity/internal/domain/issue"
	persistence "github.com/lyonbrown4d/gity/internal/infrastructure/persistence"
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
	"github.com/samber/oops"
)

type Repository struct {
	base *dbxrepo.Base[issuedomain.ProjectIssueLabel, dbschema.ProjectIssueLabelSchemaDef]
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[issuedomain.ProjectIssueLabel](db, dbschema.ProjectIssueLabelSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectIssueLabelRepository(repo *Repository) issueports.ProjectIssueLabelRepository {
	return repo
}

func (r *Repository) ListByIssueID(ctx context.Context, issueID int64) (*collectionx.List[issuedomain.ProjectIssueLabel], error) {
	query := querydsl.Select(dbschema.ProjectIssueLabelSchema.AllColumns().Values()...).
		From(dbschema.ProjectIssueLabelSchema).
		Where(dbschema.ProjectIssueLabelSchema.ProjectIssueID.Eq(issueID)).
		OrderBy(dbschema.ProjectIssueLabelSchema.Name.Asc(), dbschema.ProjectIssueLabelSchema.ID.Asc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) ReplaceByIssueID(ctx context.Context, issueID int64, labels []issueports.ProjectIssueLabelInput) (*collectionx.List[issuedomain.ProjectIssueLabel], error) {
	normalizedLabels := uniqueLabels(labels)
	var issueLabels *collectionx.List[issuedomain.ProjectIssueLabel]
	err := r.base.InTx(ctx, nil, func(_ *dbx.Tx, repo *dbxrepo.Base[issuedomain.ProjectIssueLabel, dbschema.ProjectIssueLabelSchemaDef]) error {
		listed, err := replaceIssueLabels(ctx, repo, issueID, normalizedLabels)
		if err != nil {
			return err
		}
		issueLabels = listed
		return nil
	})
	if err != nil {
		return nil, oops.In("persistence.project_issue_label").With("issue_id", issueID).Wrapf(err, "replace issue labels")
	}
	return persistence.Many(issueLabels, nil)
}

func replaceIssueLabels(ctx context.Context, repo *dbxrepo.Base[issuedomain.ProjectIssueLabel, dbschema.ProjectIssueLabelSchemaDef], issueID int64, labels []issueports.ProjectIssueLabelInput) (*collectionx.List[issuedomain.ProjectIssueLabel], error) {
	if err := deleteIssueLabels(ctx, repo, issueID); err != nil {
		return nil, err
	}
	if err := insertIssueLabels(ctx, repo, issueID, labels); err != nil {
		return nil, err
	}
	return listIssueLabels(ctx, repo, issueID)
}

func deleteIssueLabels(ctx context.Context, repo *dbxrepo.Base[issuedomain.ProjectIssueLabel, dbschema.ProjectIssueLabelSchemaDef], issueID int64) error {
	deleteQuery := querydsl.DeleteFrom(dbschema.ProjectIssueLabelSchema).
		Where(dbschema.ProjectIssueLabelSchema.ProjectIssueID.Eq(issueID))
	if _, err := repo.Delete(ctx, deleteQuery); err != nil {
		return oops.In("persistence.project_issue_label").With("issue_id", issueID).Wrapf(err, "delete issue labels")
	}
	return nil
}

func insertIssueLabels(ctx context.Context, repo *dbxrepo.Base[issuedomain.ProjectIssueLabel, dbschema.ProjectIssueLabelSchemaDef], issueID int64, labels []issueports.ProjectIssueLabelInput) error {
	if len(labels) == 0 {
		return nil
	}
	now := time.Now().UTC()
	items := make([]*issuedomain.ProjectIssueLabel, 0, len(labels))
	for _, label := range labels {
		items = append(items, &issuedomain.ProjectIssueLabel{
			ProjectIssueID: issueID,
			Name:           label.Name,
			Color:          label.Color,
			CreatedAt:      now,
			UpdatedAt:      now,
		})
	}
	if err := repo.CreateMany(ctx, items...); err != nil {
		return oops.In("persistence.project_issue_label").With("issue_id", issueID).Wrapf(err, "insert issue labels")
	}
	return nil
}

func listIssueLabels(ctx context.Context, repo *dbxrepo.Base[issuedomain.ProjectIssueLabel, dbschema.ProjectIssueLabelSchemaDef], issueID int64) (*collectionx.List[issuedomain.ProjectIssueLabel], error) {
	query := querydsl.Select(dbschema.ProjectIssueLabelSchema.AllColumns().Values()...).
		From(dbschema.ProjectIssueLabelSchema).
		Where(dbschema.ProjectIssueLabelSchema.ProjectIssueID.Eq(issueID)).
		OrderBy(dbschema.ProjectIssueLabelSchema.Name.Asc(), dbschema.ProjectIssueLabelSchema.ID.Asc())
	listed, err := repo.List(ctx, query)
	if err != nil {
		return nil, oops.In("persistence.project_issue_label").With("issue_id", issueID).Wrapf(err, "list issue labels")
	}
	return listed, nil
}

func uniqueLabels(labels []issueports.ProjectIssueLabelInput) []issueports.ProjectIssueLabelInput {
	seen := setx.NewOrderedSetWithCapacity[string](len(labels))
	items := make([]issueports.ProjectIssueLabelInput, 0, len(labels))
	for _, label := range labels {
		name := strings.TrimSpace(label.Name)
		if name == "" || seen.Contains(name) {
			continue
		}
		seen.Add(name)
		items = append(items, issueports.ProjectIssueLabelInput{Name: name, Color: strings.TrimSpace(label.Color)})
	}
	return items
}
