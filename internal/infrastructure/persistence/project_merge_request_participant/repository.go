package projectmergerequestparticipant

import (
	"context"
	"time"

	mergeports "github.com/DaiYuANg/gity/internal/application/ports"
	mergedomain "github.com/DaiYuANg/gity/internal/domain/merge"
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
	base *dbxrepo.Base[mergedomain.ProjectMergeRequestParticipant, dbschema.ProjectMergeRequestParticipantSchemaDef]
}

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[mergedomain.ProjectMergeRequestParticipant](db, dbschema.ProjectMergeRequestParticipantSchema, dbxrepo.WithKeyNotFoundAsError(true)),
	}, nil
}

func NewProjectMergeRequestParticipantRepository(repo *Repository) mergeports.ProjectMergeRequestParticipantRepository {
	return repo
}

func (r *Repository) ListByMergeRequestID(ctx context.Context, mergeRequestID int64) (*collectionx.List[mergedomain.ProjectMergeRequestParticipant], error) {
	query := querydsl.Select(dbschema.ProjectMergeRequestParticipantSchema.AllColumns().Values()...).
		From(dbschema.ProjectMergeRequestParticipantSchema).
		Where(dbschema.ProjectMergeRequestParticipantSchema.MergeRequestID.Eq(mergeRequestID)).
		OrderBy(dbschema.ProjectMergeRequestParticipantSchema.Role.Asc(), dbschema.ProjectMergeRequestParticipantSchema.ID.Asc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) ReplaceByMergeRequestAndRole(ctx context.Context, mergeRequestID int64, role string, userIDs []int64) (*collectionx.List[mergedomain.ProjectMergeRequestParticipant], error) {
	role = mergedomain.NormalizeProjectMergeRequestParticipantRole(role)
	if role == "" {
		return nil, oops.In("persistence.merge_request_participant").New("invalid merge request participant role")
	}
	orderedUserIDs := uniquePositiveInt64s(userIDs)
	var participants *collectionx.List[mergedomain.ProjectMergeRequestParticipant]
	err := r.base.InTx(ctx, nil, func(_ *dbx.Tx, repo *dbxrepo.Base[mergedomain.ProjectMergeRequestParticipant, dbschema.ProjectMergeRequestParticipantSchemaDef]) error {
		deleteQuery := querydsl.DeleteFrom(dbschema.ProjectMergeRequestParticipantSchema).
			Where(querydsl.And(
				dbschema.ProjectMergeRequestParticipantSchema.MergeRequestID.Eq(mergeRequestID),
				dbschema.ProjectMergeRequestParticipantSchema.Role.Eq(role),
			))
		if _, err := repo.Delete(ctx, deleteQuery); err != nil {
			return oops.In("persistence.merge_request_participant").With("merge_request_id", mergeRequestID, "role", role).Wrapf(err, "delete merge request participants")
		}
		now := time.Now().UTC()
		items := make([]*mergedomain.ProjectMergeRequestParticipant, 0, len(orderedUserIDs))
		for _, userID := range orderedUserIDs {
			items = append(items, &mergedomain.ProjectMergeRequestParticipant{
				MergeRequestID: mergeRequestID,
				UserID:         userID,
				Role:           role,
				CreatedAt:      now,
				UpdatedAt:      now,
			})
		}
		if err := repo.CreateMany(ctx, items...); err != nil {
			return oops.In("persistence.merge_request_participant").With("merge_request_id", mergeRequestID, "role", role).Wrapf(err, "insert merge request participants")
		}
		query := querydsl.Select(dbschema.ProjectMergeRequestParticipantSchema.AllColumns().Values()...).
			From(dbschema.ProjectMergeRequestParticipantSchema).
			Where(querydsl.And(
				dbschema.ProjectMergeRequestParticipantSchema.MergeRequestID.Eq(mergeRequestID),
				dbschema.ProjectMergeRequestParticipantSchema.Role.Eq(role),
			)).
			OrderBy(dbschema.ProjectMergeRequestParticipantSchema.ID.Asc())
		listed, err := repo.List(ctx, query)
		if err != nil {
			return oops.In("persistence.merge_request_participant").With("merge_request_id", mergeRequestID, "role", role).Wrapf(err, "list merge request participants")
		}
		participants = listed
		return nil
	})
	if err != nil {
		return nil, oops.In("persistence.merge_request_participant").With("merge_request_id", mergeRequestID, "role", role).Wrapf(err, "replace merge request participants")
	}
	return persistence.Many(participants, nil)
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
