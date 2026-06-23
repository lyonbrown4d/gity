package organization

import (
	setx "github.com/arcgolabs/collectionx/set"
	organizationports "github.com/lyonbrown4d/gity/internal/application/ports"
	"log/slog"
)

var organizationMemberRoles = setx.NewSet("guest", "reporter", "developer", "maintainer", "owner")
var organizationVisibilities = setx.NewSet("private", "internal", "public")
var organizationManageRoles = setx.NewSet("owner")
var organizationProjectCreateRoles = setx.NewSet("maintainer", "owner")

type Service struct {
	logger     *slog.Logger
	repo       organizationports.OrganizationRepository
	memberRepo organizationports.OrganizationMemberRepository
	userRepo   organizationports.UserRepository
}

type CreateInput struct {
	Name        string `json:"name"`
	PathKey     string `json:"path_key"`
	OwnerUserID int64  `json:"owner_user_id"`
	Description string `json:"description"`
	Visibility  string `json:"visibility"`
}

type UpdateInput struct {
	Name        *string `json:"name"`
	PathKey     *string `json:"path_key"`
	Description *string `json:"description"`
	Visibility  *string `json:"visibility"`
}

type AddMemberInput struct {
	UserID int64  `json:"user_id"`
	Role   string `json:"role"`
}

type MemberView struct {
	ID          int64  `json:"id"`
	UserID      int64  `json:"user_id"`
	Username    string `json:"username"`
	DisplayName string `json:"display_name"`
	Email       string `json:"email"`
	Role        string `json:"role"`
}

type Dependencies struct {
	Logger     *slog.Logger
	Repo       organizationports.OrganizationRepository
	MemberRepo organizationports.OrganizationMemberRepository
	UserRepo   organizationports.UserRepository
}

func NewDependencies(logger *slog.Logger, repo organizationports.OrganizationRepository, memberRepo organizationports.OrganizationMemberRepository, userRepo organizationports.UserRepository) Dependencies {
	return Dependencies{Logger: logger, Repo: repo, MemberRepo: memberRepo, UserRepo: userRepo}
}

func NewServiceWithDependencies(dependencies Dependencies) *Service {
	return &Service{logger: dependencies.Logger, repo: dependencies.Repo, memberRepo: dependencies.MemberRepo, userRepo: dependencies.UserRepo}
}

func NewService(logger *slog.Logger, repo organizationports.OrganizationRepository, memberRepo organizationports.OrganizationMemberRepository, userRepo organizationports.UserRepository) *Service {
	return NewServiceWithDependencies(NewDependencies(logger, repo, memberRepo, userRepo))
}
