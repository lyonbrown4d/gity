package namespace

import (
	"context"
	"errors"
	namespaceports "github.com/DaiYuANg/gity/internal/application/ports"
	namespacedomain "github.com/DaiYuANg/gity/internal/domain/namespace"
	collectionx "github.com/arcgolabs/collectionx/list"
	setx "github.com/arcgolabs/collectionx/set"
	"github.com/samber/oops"
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
	items, err := s.repo.List(ctx)
	if err != nil {
		return nil, oops.In("namespace").Wrapf(err, "list namespaces")
	}
	return items, nil
}

func (s *Service) GetByID(ctx context.Context, id int64) (namespacedomain.Namespace, error) {
	item, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return namespacedomain.Namespace{}, oops.In("namespace").With("namespace_id", id).Wrapf(err, "load namespace")
	}
	return item, nil
}

func (s *Service) Create(ctx context.Context, input CreateInput) (namespacedomain.Namespace, error) {
	if strings.TrimSpace(input.Name) == "" {
		return namespacedomain.Namespace{}, oops.In("namespace").New("namespace name is required")
	}
	if strings.TrimSpace(input.PathKey) == "" {
		return namespacedomain.Namespace{}, oops.In("namespace").With("name", input.Name).New("namespace path_key is required")
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
		return namespacedomain.Namespace{}, oops.In("namespace").With("kind", input.Kind, "name", input.Name, "path_key", input.PathKey).Wrapf(err, "create namespace")
	}
	if input.OwnerUserID > 0 {
		if _, err := s.AddMember(ctx, item.ID, AddMemberInput{UserID: input.OwnerUserID, Role: "owner"}); err != nil {
			return namespacedomain.Namespace{}, oops.In("namespace").With("namespace_id", item.ID, "owner_user_id", input.OwnerUserID).Wrapf(err, "add namespace owner")
		}
	}
	return item, nil
}

func (s *Service) Update(ctx context.Context, id int64, input UpdateInput) (namespacedomain.Namespace, error) {
	if input.Name != nil && strings.TrimSpace(*input.Name) == "" {
		return namespacedomain.Namespace{}, oops.In("namespace").With("namespace_id", id).New("namespace name is required")
	}
	if input.PathKey != nil && strings.TrimSpace(*input.PathKey) == "" {
		return namespacedomain.Namespace{}, oops.In("namespace").With("namespace_id", id).New("namespace path_key is required")
	}
	if err := s.repo.UpdateByID(ctx, id, namespaceports.UpdateNamespaceInput{
		Kind:        input.Kind,
		Name:        input.Name,
		PathKey:     input.PathKey,
		Description: input.Description,
	}); err != nil {
		return namespacedomain.Namespace{}, oops.In("namespace").With("namespace_id", id).Wrapf(err, "update namespace")
	}
	item, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return namespacedomain.Namespace{}, oops.In("namespace").With("namespace_id", id).Wrapf(err, "reload namespace")
	}
	return item, nil
}

func (s *Service) Delete(ctx context.Context, id int64) error {
	if id <= 0 {
		return oops.In("namespace").With("namespace_id", id).New("namespace id is required")
	}
	if err := s.repo.DeleteByID(ctx, id); err != nil {
		return oops.In("namespace").With("namespace_id", id).Wrapf(err, "delete namespace")
	}
	return nil
}

func (s *Service) ListMembers(ctx context.Context, namespaceID int64) ([]MemberView, error) {
	if _, err := s.repo.GetByID(ctx, namespaceID); err != nil {
		return nil, oops.In("namespace").With("namespace_id", namespaceID).Wrapf(err, "load namespace")
	}
	members, err := s.memberRepo.ListByNamespaceID(ctx, namespaceID)
	if err != nil {
		return nil, oops.In("namespace").With("namespace_id", namespaceID).Wrapf(err, "list namespace members")
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
		return MemberView{}, oops.In("namespace").With("namespace_id", namespaceID).New("namespace id is required")
	}
	if input.UserID <= 0 {
		return MemberView{}, oops.In("namespace").With("namespace_id", namespaceID, "user_id", input.UserID).New("user_id is required")
	}
	role := strings.TrimSpace(input.Role)
	if role == "" {
		role = "developer"
	}
	if role == "member" {
		role = "developer"
	}
	if !namespaceMemberRoles.Contains(role) {
		return MemberView{}, oops.In("namespace").With("namespace_id", namespaceID, "user_id", input.UserID, "role", role).New("unsupported namespace member role")
	}
	if _, err := s.repo.GetByID(ctx, namespaceID); err != nil {
		return MemberView{}, oops.In("namespace").With("namespace_id", namespaceID).Wrapf(err, "load namespace")
	}
	user, err := s.userRepo.GetByID(ctx, input.UserID)
	if err != nil {
		return MemberView{}, oops.In("namespace").With("namespace_id", namespaceID, "user_id", input.UserID).Wrapf(err, "load namespace member user")
	}
	if _, existingErr := s.memberRepo.FindByNamespaceAndUser(ctx, namespaceID, input.UserID); existingErr == nil {
		return MemberView{}, oops.In("namespace").With("namespace_id", namespaceID, "user_id", input.UserID).New("namespace member already exists")
	} else if !errors.Is(existingErr, namespaceports.ErrNotFound) {
		return MemberView{}, oops.In("namespace").With("namespace_id", namespaceID, "user_id", input.UserID).Wrapf(existingErr, "check namespace member")
	}
	member, err := s.memberRepo.Create(ctx, namespaceports.CreateNamespaceMemberInput{
		NamespaceID: namespaceID,
		UserID:      input.UserID,
		Role:        role,
	})
	if err != nil {
		return MemberView{}, oops.In("namespace").With("namespace_id", namespaceID, "user_id", input.UserID, "role", role).Wrapf(err, "create namespace member")
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
