package project

import (
	"context"
	"errors"
	"strings"

	collectionx "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	"github.com/samber/oops"
)

var projectMemberRoles = setx.NewSet("guest", "reporter", "developer", "maintainer", "owner")

type MemberService struct {
	projectRepo gitports.ProjectRepository
	userRepo    gitports.UserRepository
	memberRepo  gitports.ProjectMemberRepository
}

type MemberInput struct {
	UserID int64  `json:"user_id"`
	Role   string `json:"role"`
}

type MemberView struct {
	ID          int64  `json:"id"`
	ProjectID   int64  `json:"project_id"`
	UserID      int64  `json:"user_id"`
	Username    string `json:"username"`
	DisplayName string `json:"display_name"`
	Email       string `json:"email"`
	Role        string `json:"role"`
	Source      string `json:"source"`
}

func NewMemberService(projectRepo gitports.ProjectRepository, userRepo gitports.UserRepository, memberRepo gitports.ProjectMemberRepository) *MemberService {
	return &MemberService{projectRepo: projectRepo, userRepo: userRepo, memberRepo: memberRepo}
}

func (s *MemberService) ListMembers(ctx context.Context, projectID int64) ([]MemberView, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	items, err := s.memberRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, oops.In("project").With("project_id", projectID).Wrapf(err, "list project members")
	}
	return collectionx.FilterMapList(items, func(_ int, item projectdomain.ProjectMember) (MemberView, bool) {
		view, buildErr := s.buildMemberView(ctx, item)
		return view, buildErr == nil
	}).Values(), nil
}

func (s *MemberService) UpsertMember(ctx context.Context, projectID int64, input MemberInput) (MemberView, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return MemberView{}, apperror.NotFound("project not found", err)
	}
	if input.UserID <= 0 {
		return MemberView{}, apperror.BadRequest("project member user_id is required", oops.In("project").With("project_id", projectID, "user_id", input.UserID).New("project member user_id is required"))
	}
	role := normalizeProjectMemberRole(input.Role)
	if !projectMemberRoles.Contains(role) {
		return MemberView{}, apperror.BadRequest("unsupported project member role", oops.In("project").With("project_id", projectID, "user_id", input.UserID, "role", role).New("unsupported project member role"))
	}
	if _, err := s.userRepo.GetByID(ctx, input.UserID); err != nil {
		return MemberView{}, apperror.NotFound("project member user not found", err)
	}
	member, err := s.memberRepo.FindByProjectAndUser(ctx, projectID, input.UserID)
	if err == nil {
		if updateErr := s.memberRepo.UpdateRoleByID(ctx, member.ID, role); updateErr != nil {
			return MemberView{}, oops.In("project").With("project_id", projectID, "user_id", input.UserID, "role", role).Wrapf(updateErr, "update project member")
		}
		member.Role = role
		return s.buildMemberView(ctx, member)
	}
	if !errors.Is(err, gitports.ErrNotFound) {
		return MemberView{}, oops.In("project").With("project_id", projectID, "user_id", input.UserID).Wrapf(err, "load project member")
	}
	created, err := s.memberRepo.Create(ctx, gitports.CreateProjectMemberInput{ProjectID: projectID, UserID: input.UserID, Role: role})
	if err != nil {
		return MemberView{}, oops.In("project").With("project_id", projectID, "user_id", input.UserID, "role", role).Wrapf(err, "create project member")
	}
	return s.buildMemberView(ctx, created)
}

func (s *MemberService) DeleteMember(ctx context.Context, projectID, userID int64) error {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return apperror.NotFound("project not found", err)
	}
	if userID <= 0 {
		return apperror.BadRequest("project member user_id is required", oops.In("project").With("project_id", projectID, "user_id", userID).New("project member user_id is required"))
	}
	if err := s.memberRepo.DeleteByProjectAndUser(ctx, projectID, userID); err != nil {
		return oops.In("project").With("project_id", projectID, "user_id", userID).Wrapf(err, "delete project member")
	}
	return nil
}

func (s *MemberService) buildMemberView(ctx context.Context, item projectdomain.ProjectMember) (MemberView, error) {
	user, err := s.userRepo.GetByID(ctx, item.UserID)
	if err != nil {
		return MemberView{}, err
	}
	return MemberView{
		ID:          item.ID,
		ProjectID:   item.ProjectID,
		UserID:      item.UserID,
		Username:    user.Username,
		DisplayName: user.DisplayName,
		Email:       user.Email,
		Role:        item.Role,
		Source:      "project",
	}, nil
}

func normalizeProjectMemberRole(value string) string {
	role := strings.TrimSpace(strings.ToLower(value))
	if role == "" {
		return "developer"
	}
	return role
}
