package user_test

import (
	"context"
	"slices"
	"strings"
	"testing"

	collectionx "github.com/arcgolabs/collectionx/list"
	identityports "github.com/lyonbrown4d/gity/internal/application/ports"
	userservice "github.com/lyonbrown4d/gity/internal/application/user"
	identity "github.com/lyonbrown4d/gity/internal/domain/identity"
)

func TestServiceRejectsDemotingLastSuperAdmin(t *testing.T) {
	t.Parallel()

	repo := newMemoryUserRepository(identity.User{ID: 1, Username: "root", IsSuperAdmin: 1})
	service := userservice.NewService(nil, repo, nil)
	demote := false

	if _, err := service.Update(context.Background(), 1, userservice.UpdateInput{IsSuperAdmin: &demote}); err == nil {
		t.Fatal("expected demoting the last super admin to fail")
	}
	user, err := repo.GetByID(context.Background(), 1)
	user = mustUser(t, user, err)
	if user.IsSuperAdmin == 0 {
		t.Fatal("expected last super admin flag to be preserved")
	}
}

func TestServiceAllowsDemotingSuperAdminWhenAnotherExists(t *testing.T) {
	t.Parallel()

	repo := newMemoryUserRepository(
		identity.User{ID: 1, Username: "root", IsSuperAdmin: 1},
		identity.User{ID: 2, Username: "backup", IsSuperAdmin: 1},
	)
	service := userservice.NewService(nil, repo, nil)
	demote := false

	if _, err := service.Update(context.Background(), 1, userservice.UpdateInput{IsSuperAdmin: &demote}); err != nil {
		t.Fatalf("expected demoting one of multiple super admins to succeed: %v", err)
	}
	user, err := repo.GetByID(context.Background(), 1)
	user = mustUser(t, user, err)
	if user.IsSuperAdmin != 0 {
		t.Fatal("expected super admin flag to be removed")
	}
}

func TestServiceRejectsDeletingLastSuperAdmin(t *testing.T) {
	t.Parallel()

	repo := newMemoryUserRepository(identity.User{ID: 1, Username: "root", IsSuperAdmin: 1})
	service := userservice.NewService(nil, repo, nil)

	if err := service.Delete(context.Background(), 1); err == nil {
		t.Fatal("expected deleting the last super admin to fail")
	}
	user, err := repo.GetByID(context.Background(), 1)
	user = mustUser(t, user, err)
	if user.IsSuperAdmin == 0 {
		t.Fatal("expected last super admin user to remain")
	}
}

type memoryUserRepository struct {
	users map[int64]identity.User
	next  int64
}

func newMemoryUserRepository(users ...identity.User) *memoryUserRepository {
	repo := &memoryUserRepository{users: map[int64]identity.User{}, next: 1}
	for _, user := range users {
		if user.ID <= 0 {
			user.ID = repo.next
		}
		repo.users[user.ID] = user
		if user.ID >= repo.next {
			repo.next = user.ID + 1
		}
	}
	return repo
}

func (r *memoryUserRepository) List(context.Context) (*collectionx.List[identity.User], error) {
	ids := make([]int64, 0, len(r.users))
	for id := range r.users {
		ids = append(ids, id)
	}
	slices.Sort(ids)
	users := make([]identity.User, 0, len(ids))
	for _, id := range ids {
		users = append(users, r.users[id])
	}
	return collectionx.NewList(users...), nil
}

func (r *memoryUserRepository) GetByID(_ context.Context, id int64) (identity.User, error) {
	user, ok := r.users[id]
	if !ok {
		return identity.User{}, identityports.ErrNotFound
	}
	return user, nil
}

func (r *memoryUserRepository) GetByUsername(_ context.Context, username string) (identity.User, error) {
	username = strings.TrimSpace(username)
	for _, user := range r.users {
		if user.Username == username {
			return user, nil
		}
	}
	return identity.User{}, identityports.ErrNotFound
}

func (r *memoryUserRepository) Create(_ context.Context, input identityports.CreateUserInput) (identity.User, error) {
	user := identity.User{
		ID:           r.next,
		Username:     strings.TrimSpace(input.Username),
		DisplayName:  strings.TrimSpace(input.DisplayName),
		Email:        strings.TrimSpace(input.Email),
		IsSuperAdmin: boolAsInt(input.IsSuperAdmin),
	}
	r.next++
	r.users[user.ID] = user
	return user, nil
}

func (r *memoryUserRepository) UpdateByID(_ context.Context, id int64, input identityports.UpdateUserInput) error {
	user, ok := r.users[id]
	if !ok {
		return identityports.ErrNotFound
	}
	if input.Username != nil {
		user.Username = strings.TrimSpace(*input.Username)
	}
	if input.DisplayName != nil {
		user.DisplayName = strings.TrimSpace(*input.DisplayName)
	}
	if input.Email != nil {
		user.Email = strings.TrimSpace(*input.Email)
	}
	if input.IsSuperAdmin != nil {
		user.IsSuperAdmin = boolAsInt(*input.IsSuperAdmin)
	}
	r.users[id] = user
	return nil
}

func (r *memoryUserRepository) DeleteByID(_ context.Context, id int64) error {
	if _, ok := r.users[id]; !ok {
		return identityports.ErrNotFound
	}
	delete(r.users, id)
	return nil
}

func boolAsInt(value bool) int {
	if value {
		return 1
	}
	return 0
}

func mustUser(t *testing.T, user identity.User, err error) identity.User {
	t.Helper()
	if err != nil {
		t.Fatalf("expected user lookup to succeed: %v", err)
	}
	return user
}

var _ identityports.UserRepository = (*memoryUserRepository)(nil)
