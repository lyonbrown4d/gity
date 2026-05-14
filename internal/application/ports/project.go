package ports

import (
	"context"
	"time"

	collectionx "github.com/arcgolabs/collectionx/list"
	organizationdomain "github.com/lyonbrown4d/gity/internal/domain/organization"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
)

type ProjectRepository interface {
	List(ctx context.Context, organizationID *int64) (*collectionx.List[projectdomain.Project], error)
	GetByID(ctx context.Context, id int64) (projectdomain.Project, error)
	GetIncludingDeletedByID(ctx context.Context, id int64) (projectdomain.Project, error)
	GetByFullPath(ctx context.Context, fullPath string) (projectdomain.Project, error)
	Create(ctx context.Context, input CreateProjectInput, organization organizationdomain.Organization) (projectdomain.Project, error)
	MarkPendingDeleteByID(ctx context.Context, id int64, deletedAt time.Time) error
	DeleteByID(ctx context.Context, id int64) error
}

type ProjectBatchRepository interface {
	Batch(ctx context.Context, organizationID *int64, size int, handle func(*collectionx.List[projectdomain.Project]) error) error
}

type ProjectBranchProtectionRepository interface {
	ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[projectdomain.ProjectBranchProtection], error)
	GetByProjectAndBranch(ctx context.Context, projectID int64, branchName string) (projectdomain.ProjectBranchProtection, error)
	MatchByProjectAndBranch(ctx context.Context, projectID int64, branchName string) (projectdomain.ProjectBranchProtection, error)
	Protect(ctx context.Context, projectID int64, branchName string) (projectdomain.ProjectBranchProtection, error)
	Upsert(ctx context.Context, projectID int64, input UpsertProjectBranchProtectionInput) (projectdomain.ProjectBranchProtection, error)
	Unprotect(ctx context.Context, projectID int64, branchName string) error
}

type ProjectMemberRepository interface {
	ListByProjectID(ctx context.Context, projectID int64) (*collectionx.List[projectdomain.ProjectMember], error)
	FindByProjectAndUser(ctx context.Context, projectID, userID int64) (projectdomain.ProjectMember, error)
	Create(ctx context.Context, input CreateProjectMemberInput) (projectdomain.ProjectMember, error)
	UpdateRoleByID(ctx context.Context, id int64, role string) error
	DeleteByProjectAndUser(ctx context.Context, projectID, userID int64) error
}

type CreateProjectInput struct {
	OrganizationID int64
	Name           string
	PathKey        string
	Visibility     string
	Description    string
	DefaultBranch  string
}

type CreateProjectMemberInput struct {
	ProjectID int64
	UserID    int64
	Role      string
}

type UpsertProjectBranchProtectionInput struct {
	BranchName             string
	RuleType               string
	PushAccessLevel        string
	MergeAccessLevel       string
	RequireMergeRequest    bool
	RequirePipelineSuccess bool
	AllowForcePush         bool
	AllowDelete            bool
}
