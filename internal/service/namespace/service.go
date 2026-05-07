package namespace

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"strings"

	"github.com/DaiYuANg/gity/internal/entity"
	namespacerepo "github.com/DaiYuANg/gity/internal/repository/namespace"
	namespacememberrepo "github.com/DaiYuANg/gity/internal/repository/namespacemember"
	userrepo "github.com/DaiYuANg/gity/internal/repository/user"
	collectionx "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	dbxrepo "github.com/arcgolabs/dbx/repository"
)

var namespaceMemberRoles = setx.NewSet("guest", "reporter", "developer", "maintainer", "owner")

type Service struct {
	logger     *slog.Logger
	repo       *namespacerepo.Repository
	memberRepo *namespacememberrepo.Repository
	userRepo   *userrepo.Repository
}

type CreateInput struct {
	Kind        string `json:"kind"`
	Name        string `json:"name"`
	PathKey     string `json:"path_key"`
	OwnerUserID int64  `json:"owner_user_id"`
	Description string `json:"description"`
}

type UpdateInput struct {
	Kind        *string `json:"kind"`
	Name        *string `json:"name"`
	PathKey     *string `json:"path_key"`
	Description *string `json:"description"`
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

func NewService(logger *slog.Logger, repo *namespacerepo.Repository, memberRepo *namespacememberrepo.Repository, userRepo *userrepo.Repository) *Service {
	return &Service{logger: logger, repo: repo, memberRepo: memberRepo, userRepo: userRepo}
}

func (s *Service) List(ctx context.Context) (*collectionx.List[entity.Namespace], error) {
	return s.repo.List(ctx)
}

func (s *Service) GetByID(ctx context.Context, id int64) (entity.Namespace, error) {
	return s.repo.GetByID(ctx, id)
}

func (s *Service) Create(ctx context.Context, input CreateInput) (entity.Namespace, error) {
	if strings.TrimSpace(input.Name) == "" {
		return entity.Namespace{}, fmt.Errorf("namespace name is required")
	}
	if strings.TrimSpace(input.PathKey) == "" {
		return entity.Namespace{}, fmt.Errorf("namespace path_key is required")
	}
	if strings.TrimSpace(input.Kind) == "" {
		input.Kind = "group"
	}
	item, err := s.repo.Create(ctx, namespacerepo.CreateInput{
		Kind:        input.Kind,
		Name:        input.Name,
		PathKey:     input.PathKey,
		Description: input.Description,
	})
	if err != nil {
		return entity.Namespace{}, err
	}
	if input.OwnerUserID > 0 {
		if _, err := s.AddMember(ctx, item.ID, AddMemberInput{UserID: input.OwnerUserID, Role: "owner"}); err != nil {
			return entity.Namespace{}, err
		}
	}
	return item, nil
}

func (s *Service) Update(ctx context.Context, id int64, input UpdateInput) (entity.Namespace, error) {
	if input.Name != nil && strings.TrimSpace(*input.Name) == "" {
		return entity.Namespace{}, fmt.Errorf("namespace name is required")
	}
	if input.PathKey != nil && strings.TrimSpace(*input.PathKey) == "" {
		return entity.Namespace{}, fmt.Errorf("namespace path_key is required")
	}
	if err := s.repo.UpdateByID(ctx, id, namespacerepo.UpdateInput{
		Kind:        input.Kind,
		Name:        input.Name,
		PathKey:     input.PathKey,
		Description: input.Description,
	}); err != nil {
		return entity.Namespace{}, err
	}
	return s.repo.GetByID(ctx, id)
}

func (s *Service) Delete(ctx context.Context, id int64) error {
	if id <= 0 {
		return fmt.Errorf("namespace id is required")
	}
	return s.repo.DeleteByID(ctx, id)
}

func (s *Service) ListMembers(ctx context.Context, namespaceID int64) ([]MemberView, error) {
	if _, err := s.repo.GetByID(ctx, namespaceID); err != nil {
		return nil, fmt.Errorf("load namespace: %w", err)
	}
	members, err := s.memberRepo.ListByNamespaceID(ctx, namespaceID)
	if err != nil {
		return nil, err
	}
	views := make([]MemberView, 0, members.Len())
	members.Range(func(_ int, item entity.NamespaceMember) bool {
		user, userErr := s.userRepo.GetByID(ctx, item.UserID)
		if userErr != nil {
			return true
		}
		views = append(views, MemberView{
			ID:          item.ID,
			UserID:      item.UserID,
			Username:    user.Username,
			DisplayName: user.DisplayName,
			Email:       user.Email,
			Role:        item.Role,
		})
		return true
	})
	return views, nil
}

func (s *Service) AddMember(ctx context.Context, namespaceID int64, input AddMemberInput) (MemberView, error) {
	if namespaceID <= 0 {
		return MemberView{}, fmt.Errorf("namespace id is required")
	}
	if input.UserID <= 0 {
		return MemberView{}, fmt.Errorf("user_id is required")
	}
	role := strings.TrimSpace(input.Role)
	if role == "" {
		role = "developer"
	}
	if role == "member" {
		role = "developer"
	}
	if !namespaceMemberRoles.Contains(role) {
		return MemberView{}, fmt.Errorf("unsupported namespace member role: %s", role)
	}
	if _, err := s.repo.GetByID(ctx, namespaceID); err != nil {
		return MemberView{}, fmt.Errorf("load namespace: %w", err)
	}
	user, err := s.userRepo.GetByID(ctx, input.UserID)
	if err != nil {
		return MemberView{}, fmt.Errorf("load user: %w", err)
	}
	if _, err := s.memberRepo.FindByNamespaceAndUser(ctx, namespaceID, input.UserID); err == nil {
		return MemberView{}, fmt.Errorf("namespace member already exists")
	} else if err != nil && !errors.Is(err, dbxrepo.ErrNotFound) {
		return MemberView{}, err
	}
	member, err := s.memberRepo.Create(ctx, namespacememberrepo.CreateInput{
		NamespaceID: namespaceID,
		UserID:      input.UserID,
		Role:        role,
	})
	if err != nil {
		return MemberView{}, err
	}
	return MemberView{
		ID:          member.ID,
		UserID:      user.ID,
		Username:    user.Username,
		DisplayName: user.DisplayName,
		Email:       user.Email,
		Role:        member.Role,
	}, nil
}
