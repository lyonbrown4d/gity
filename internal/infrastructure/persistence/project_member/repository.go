package projectmember

import (
	"context"
	"fmt"
	"strings"
	"time"

	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx"
	"github.com/arcgolabs/dbx/querydsl"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	projectports "github.com/lyonbrown4d/gity/internal/application/ports"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	persistence "github.com/lyonbrown4d/gity/internal/infrastructure/persistence"
	dbaudit "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_audit"
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
)

type Repository struct {
	base *dbxrepo.Base[projectdomain.ProjectMember, dbschema.ProjectMemberSchemaDef]
}

type CreateInput = projectports.CreateProjectMemberInput

func NewRepository(db *dbx.DB) (*Repository, error) {
	return &Repository{
		base: dbxrepo.NewWithOptions[projectdomain.ProjectMember](
			db,
			dbschema.ProjectMemberSchema,
			dbxrepo.WithKeyNotFoundAsError(true),
			dbxrepo.WithAuditWriter(dbaudit.ProjectMemberAudit()),
		),
	}, nil
}

func NewProjectMemberRepository(repo *Repository) projectports.ProjectMemberRepository {
	return repo
}

func (r *Repository) ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[projectdomain.ProjectMember], error) {
	query := querydsl.Select(dbschema.ProjectMemberSchema.AllColumns().Values()...).
		From(dbschema.ProjectMemberSchema).
		Where(dbschema.ProjectMemberSchema.ProjectID.Eq(projectID)).
		OrderBy(dbschema.ProjectMemberSchema.ID.Desc())
	return persistence.Many(r.base.List(ctx, query))
}

func (r *Repository) FindByProjectAndUser(ctx context.Context, projectID, userID int64) (projectdomain.ProjectMember, error) {
	return persistence.One(r.base.GetByKey(ctx, dbxrepo.Key{
		"project_id": projectID,
		"user_id":    userID,
	}))
}

func (r *Repository) Create(ctx context.Context, input CreateInput) (projectdomain.ProjectMember, error) {
	now := time.Now().UTC()
	item := projectdomain.ProjectMember{
		ProjectID: input.ProjectID,
		UserID:    input.UserID,
		Role:      strings.TrimSpace(input.Role),
		CreatedAt: now,
		UpdatedAt: now,
	}
	if err := r.base.Create(ctx, &item); err != nil {
		return projectdomain.ProjectMember{}, fmt.Errorf("insert project member: %w", err)
	}
	return item, nil
}

func (r *Repository) UpdateRoleByID(ctx context.Context, id int64, role string) error {
	if _, err := dbxrepo.PatchSet(r.base, projectMemberKey(id)).Set(
		dbschema.ProjectMemberSchema.Role.Set(strings.TrimSpace(role)),
		dbschema.ProjectMemberSchema.UpdatedAt.Set(time.Now().UTC()),
	).Apply(ctx); err != nil {
		return fmt.Errorf("update project member role: %w", err)
	}
	return nil
}

func (r *Repository) DeleteByProjectAndUser(ctx context.Context, projectID, userID int64) error {
	member, err := r.FindByProjectAndUser(ctx, projectID, userID)
	if err != nil {
		if persistence.IsNotFound(err) {
			return nil
		}
		return err
	}
	if _, err := r.base.DeleteByKeySet(ctx, projectMemberKey(member.ID)); err != nil {
		return fmt.Errorf("delete project member: %w", err)
	}
	return nil
}

func projectMemberKey(id int64) dbxrepo.TypedKeySet {
	return dbxrepo.KeySet(dbxrepo.Part(dbschema.ProjectMemberSchema.ID, id))
}
