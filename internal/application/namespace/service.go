package namespace

import (
	"context"
	"errors"
	"fmt"
	namespaceports "github.com/DaiYuANg/gity/internal/application/ports"
	namespacedomain "github.com/DaiYuANg/gity/internal/domain/namespace"
	collectionx "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	"log/slog"
	"strings"
)

var namespaceMemberRoles = setx.NewSet("guest", "reporter", "developer", "maintainer", "owner")

type Service struct {
	logger     *slog.Logger
	repo       namespaceports.NamespaceRepository
	memberRepo namespaceports.NamespaceMemberRepository
	userRepo   namespaceports.UserRepository
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

func NewService(logger *slog.Logger, repo namespaceports.NamespaceRepository, memberRepo namespaceports.NamespaceMemberRepository, userRepo namespaceports.UserRepository) *Service {
	return &Service{logger: logger, repo: repo, memberRepo: memberRepo, userRepo: userRepo}
}

func (s *Service) List(ctx context.Context) (*collectionx.List[namespacedomain.Namespace], error) {
	return s.repo.List(ctx)
}

func (s *Service) GetByID(ctx context.Context, id int64) (namespacedomain.Namespace, error) {
	return s.repo.GetByID(ctx, id)
}

func (s *Service) Create(ctx context.Context, input CreateInput) (namespacedomain.Namespace, error) {
	if strings.TrimSpace(input.Name) == "" {
		return namespacedomain.Namespace{}, errors.New("namespace name is required")
	}
	if strings.TrimSpace(input.PathKey) == "" {
		return namespacedomain.Namespace{}, errors.New("namespace path_key is required")
	}
	if strings.TrimSpace(input.Kind) == "" {
		input.Kind = "group"
	}
	item, err := s.repo.Create(ctx, namespaceports.CreateNamespaceInput{
		Kind:        input.Kind,
		Name:        input.Name,
		PathKey:     input.PathKey,
		Description: input.Description,
	})
	if err != nil {
		return namespacedomain.Namespace{}, err
	}
	if input.OwnerUserID > 0 {
		if _, err := s.AddMember(ctx, item.ID, AddMemberInput{UserID: input.OwnerUserID, Role: "owner"}); err != nil {
			return namespacedomain.Namespace{}, err
		}
	}
	return item, nil
}

func (s *Service) Update(ctx context.Context, id int64, input UpdateInput) (namespacedomain.Namespace, error) {
	if input.Name != nil && strings.TrimSpace(*input.Name) == "" {
		return namespacedomain.Namespace{}, errors.New("namespace name is required")
	}
	if input.PathKey != nil && strings.TrimSpace(*input.PathKey) == "" {
		return namespacedomain.Namespace{}, errors.New("namespace path_key is required")
	}
	if err := s.repo.UpdateByID(ctx, id, namespaceports.UpdateNamespaceInput{
		Kind:        input.Kind,
		Name:        input.Name,
		PathKey:     input.PathKey,
		Description: input.Description,
	}); err != nil {
		return namespacedomain.Namespace{}, err
	}
	return s.repo.GetByID(ctx, id)
}

func (s *Service) Delete(ctx context.Context, id int64) error {
	if id <= 0 {
		return errors.New("namespace id is required")
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
	return collectionx.FilterMapList(members, func(_ int, item namespacedomain.NamespaceMember) (MemberView, bool) {
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

func (s *Service) AddMember(ctx context.Context, namespaceID int64, input AddMemberInput) (MemberView, error) {
	if namespaceID <= 0 {
		return MemberView{}, errors.New("namespace id is required")
	}
	if input.UserID <= 0 {
		return MemberView{}, errors.New("user_id is required")
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
	if _, existingErr := s.memberRepo.FindByNamespaceAndUser(ctx, namespaceID, input.UserID); existingErr == nil {
		return MemberView{}, errors.New("namespace member already exists")
	} else if !errors.Is(existingErr, namespaceports.ErrNotFound) {
		return MemberView{}, existingErr
	}
	member, err := s.memberRepo.Create(ctx, namespaceports.CreateNamespaceMemberInput{
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
