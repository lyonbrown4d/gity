package organization

import (
	"context"
	"errors"
	collectionx "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	organizationports "github.com/lyonbrown4d/gity/internal/application/ports"
	identitydomain "github.com/lyonbrown4d/gity/internal/domain/identity"
	organizationdomain "github.com/lyonbrown4d/gity/internal/domain/organization"
	"github.com/samber/oops"
	"log/slog"
	"strings"
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

func (s *Service) List(ctx context.Context) (*collectionx.List[organizationdomain.Organization], error) {
	items, err := s.repo.List(ctx)
	if err != nil {
		return nil, oops.In("organization").Wrapf(err, "list organizations")
	}
	return items, nil
}

func (s *Service) GetByID(ctx context.Context, id int64) (organizationdomain.Organization, error) {
	item, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return organizationdomain.Organization{}, oops.In("organization").With("organization_id", id).Wrapf(err, "load organization")
	}
	return item, nil
}

func (s *Service) Create(ctx context.Context, input CreateInput) (organizationdomain.Organization, error) {
	visibility, err := validateCreateInput(input)
	if err != nil {
		return organizationdomain.Organization{}, err
	}
	if strings.TrimSpace(input.Name) == "" {
		return organizationdomain.Organization{}, oops.In("organization").New("organization name is required")
	}
	if strings.TrimSpace(input.PathKey) == "" {
		return organizationdomain.Organization{}, oops.In("organization").With("name", input.Name).New("organization path_key is required")
	}
	item, err := s.repo.Create(ctx, organizationports.CreateOrganizationInput{
		Name:        input.Name,
		PathKey:     input.PathKey,
		Description: input.Description,
		Visibility:  visibility,
	})
	if err != nil {
		return organizationdomain.Organization{}, oops.In("organization").With("name", input.Name, "path_key", input.PathKey).Wrapf(err, "create organization")
	}
	if input.OwnerUserID > 0 {
		if _, err := s.AddMember(ctx, item.ID, AddMemberInput{UserID: input.OwnerUserID, Role: "owner"}); err != nil {
			return organizationdomain.Organization{}, oops.In("organization").With("organization_id", item.ID, "owner_user_id", input.OwnerUserID).Wrapf(err, "add organization owner")
		}
	}
	return item, nil
}

func (s *Service) Update(ctx context.Context, id int64, input UpdateInput) (organizationdomain.Organization, error) {
	if input.Name != nil && strings.TrimSpace(*input.Name) == "" {
		return organizationdomain.Organization{}, oops.In("organization").With("organization_id", id).New("organization name is required")
	}
	if input.PathKey != nil && strings.TrimSpace(*input.PathKey) == "" {
		return organizationdomain.Organization{}, oops.In("organization").With("organization_id", id).New("organization path_key is required")
	}
	var visibility *string
	if input.Visibility != nil {
		normalizedVisibility, visibilityErr := normalizeVisibility(*input.Visibility, id)
		if visibilityErr != nil {
			return organizationdomain.Organization{}, visibilityErr
		}
		visibility = &normalizedVisibility
	}
	if updateErr := s.repo.UpdateByID(ctx, id, organizationports.UpdateOrganizationInput{
		Name:        input.Name,
		PathKey:     input.PathKey,
		Description: input.Description,
		Visibility:  visibility,
	}); updateErr != nil {
		return organizationdomain.Organization{}, oops.In("organization").With("organization_id", id).Wrapf(updateErr, "update organization")
	}
	item, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return organizationdomain.Organization{}, oops.In("organization").With("organization_id", id).Wrapf(err, "reload organization")
	}
	return item, nil
}

func validateCreateInput(input CreateInput) (string, error) {
	return normalizeVisibility(input.Visibility, input.Name)
}

func normalizeVisibility(value string, contextValue any) (string, error) {
	visibility := strings.TrimSpace(strings.ToLower(value))
	if visibility == "" {
		visibility = "private"
	}
	if !organizationVisibilities.Contains(visibility) {
		return "", oops.In("organization").With("context", contextValue, "visibility", visibility).New("unsupported organization visibility")
	}
	return visibility, nil
}

func (s *Service) Delete(ctx context.Context, id int64) error {
	if id <= 0 {
		return oops.In("organization").With("organization_id", id).New("organization id is required")
	}
	if err := s.repo.DeleteByID(ctx, id); err != nil {
		return oops.In("organization").With("organization_id", id).Wrapf(err, "delete organization")
	}
	return nil
}

func (s *Service) ListMembers(ctx context.Context, organizationID int64) ([]MemberView, error) {
	if _, err := s.repo.GetByID(ctx, organizationID); err != nil {
		return nil, oops.In("organization").With("organization_id", organizationID).Wrapf(err, "load organization")
	}
	members, err := s.memberRepo.ListByOrganizationID(ctx, organizationID)
	if err != nil {
		return nil, oops.In("organization").With("organization_id", organizationID).Wrapf(err, "list organization members")
	}
	return collectionx.FilterMapList(members, func(_ int, item organizationdomain.OrganizationMember) (MemberView, bool) {
		user, userErr := s.userRepo.GetByID(ctx, item.UserID)
		if userErr != nil {
			return MemberView{}, false
		}
		return MemberView{
			ID:          item.ID,
			UserID:      item.UserID,
			Username:    user.Username,
			DisplayName: user.DisplayName,
			Email:       user.Email,
			Role:        item.Role,
		}, true
	}).Values(), nil
}

func (s *Service) CanRead(ctx context.Context, item organizationdomain.Organization, userID int64) (bool, error) {
	switch strings.TrimSpace(strings.ToLower(item.Visibility)) {
	case "public", "internal":
		return true, nil
	}
	return s.isMember(ctx, item.ID, userID)
}

func (s *Service) CanManage(ctx context.Context, organizationID, userID int64) (bool, error) {
	member, err := s.memberRepo.FindByOrganizationAndUser(ctx, organizationID, userID)
	if err != nil {
		if errors.Is(err, organizationports.ErrNotFound) {
			return false, nil
		}
		return false, oops.In("organization").
			With("organization_id", organizationID, "user_id", userID).
			Wrapf(err, "load organization membership")
	}
	return organizationManageRoles.Contains(strings.TrimSpace(member.Role)), nil
}

func (s *Service) CanCreateProject(ctx context.Context, organizationID, userID int64) (bool, error) {
	member, err := s.memberRepo.FindByOrganizationAndUser(ctx, organizationID, userID)
	if err != nil {
		if errors.Is(err, organizationports.ErrNotFound) {
			return false, nil
		}
		return false, oops.In("organization").
			With("organization_id", organizationID, "user_id", userID).
			Wrapf(err, "load organization membership")
	}
	return organizationProjectCreateRoles.Contains(strings.TrimSpace(member.Role)), nil
}

func (s *Service) AddMember(ctx context.Context, organizationID int64, input AddMemberInput) (MemberView, error) {
	if organizationID <= 0 {
		return MemberView{}, oops.In("organization").With("organization_id", organizationID).New("organization id is required")
	}
	if input.UserID <= 0 {
		return MemberView{}, oops.In("organization").With("organization_id", organizationID, "user_id", input.UserID).New("user_id is required")
	}
	role := normalizeMemberRole(input.Role)
	if !organizationMemberRoles.Contains(role) {
		return MemberView{}, oops.In("organization").With("organization_id", organizationID, "user_id", input.UserID, "role", role).New("unsupported organization member role")
	}
	if _, err := s.repo.GetByID(ctx, organizationID); err != nil {
		return MemberView{}, oops.In("organization").With("organization_id", organizationID).Wrapf(err, "load organization")
	}
	user, err := s.userRepo.GetByID(ctx, input.UserID)
	if err != nil {
		return MemberView{}, oops.In("organization").With("organization_id", organizationID, "user_id", input.UserID).Wrapf(err, "load organization member user")
	}
	if memberErr := s.ensureMemberDoesNotExist(ctx, organizationID, input.UserID); memberErr != nil {
		return MemberView{}, memberErr
	}
	member, err := s.memberRepo.Create(ctx, organizationports.CreateOrganizationMemberInput{
		OrganizationID: organizationID,
		UserID:         input.UserID,
		Role:           role,
	})
	if err != nil {
		return MemberView{}, oops.In("organization").With("organization_id", organizationID, "user_id", input.UserID, "role", role).Wrapf(err, "create organization member")
	}
	return buildMemberView(member, user), nil
}

func (s *Service) isMember(ctx context.Context, organizationID, userID int64) (bool, error) {
	if _, err := s.memberRepo.FindByOrganizationAndUser(ctx, organizationID, userID); err != nil {
		if errors.Is(err, organizationports.ErrNotFound) {
			return false, nil
		}
		return false, oops.In("organization").
			With("organization_id", organizationID, "user_id", userID).
			Wrapf(err, "load organization membership")
	}
	return true, nil
}

func normalizeMemberRole(value string) string {
	role := strings.TrimSpace(value)
	if role == "" || role == "member" {
		return "developer"
	}
	return role
}

func (s *Service) ensureMemberDoesNotExist(ctx context.Context, organizationID, userID int64) error {
	if _, existingErr := s.memberRepo.FindByOrganizationAndUser(ctx, organizationID, userID); existingErr == nil {
		return oops.In("organization").With("organization_id", organizationID, "user_id", userID).New("organization member already exists")
	} else if !errors.Is(existingErr, organizationports.ErrNotFound) {
		return oops.In("organization").With("organization_id", organizationID, "user_id", userID).Wrapf(existingErr, "check organization member")
	}
	return nil
}

func buildMemberView(member organizationdomain.OrganizationMember, user identitydomain.User) MemberView {
	return MemberView{
		ID:          member.ID,
		UserID:      user.ID,
		Username:    user.Username,
		DisplayName: user.DisplayName,
		Email:       user.Email,
		Role:        member.Role,
	}
}
