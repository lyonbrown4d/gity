package user_test

import (
	"context"
	"errors"
	"slices"
	"strings"
	"testing"

	collectionx "github.com/arcgolabs/collectionx/list"
	appports "github.com/lyonbrown4d/gity/internal/application/ports"
	userservice "github.com/lyonbrown4d/gity/internal/application/user"
	identity "github.com/lyonbrown4d/gity/internal/domain/identity"
	organizationdomain "github.com/lyonbrown4d/gity/internal/domain/organization"
)

func TestLoginCreatesDefaultOrganizationForNewUser(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	userRepo := newMemoryUserRepository()
	tokenRepo := newMemoryUserAccessTokenRepository()
	organizationRepo := newMemoryOrganizationRepository()
	organizationMemberRepo := newMemoryOrganizationMemberRepository()
	service := userservice.NewServiceWithDependencies(userservice.Dependencies{
		Repo:                   userRepo,
		TokenRepo:              tokenRepo,
		OrganizationRepo:       organizationRepo,
		OrganizationMemberRepo: organizationMemberRepo,
	})

	session, err := service.Login(ctx, "Alice Owner")
	if err != nil {
		t.Fatalf("expected login to create default organization: %v", err)
	}
	assertDefaultOrganization(t, ctx, organizationRepo, organizationMemberRepo, session.User.ID, "alice-owner")

	if _, err := service.Login(ctx, "Alice Owner"); err != nil {
		t.Fatalf("expected repeated login to remain idempotent: %v", err)
	}
	organizations, err := organizationRepo.List(ctx)
	if err != nil {
		t.Fatalf("expected organizations list to succeed: %v", err)
	}
	if organizations.Len() != 1 {
		t.Fatalf("expected repeated login to keep one default organization, got %d", organizations.Len())
	}
}

func TestLoginBackfillsDefaultOrganizationForExistingUser(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	userRepo := newMemoryUserRepository(identity.User{ID: 42, Username: "existing.user", DisplayName: "Existing User"})
	tokenRepo := newMemoryUserAccessTokenRepository()
	organizationRepo := newMemoryOrganizationRepository()
	organizationMemberRepo := newMemoryOrganizationMemberRepository()
	service := userservice.NewServiceWithDependencies(userservice.Dependencies{
		Repo:                   userRepo,
		TokenRepo:              tokenRepo,
		OrganizationRepo:       organizationRepo,
		OrganizationMemberRepo: organizationMemberRepo,
	})

	session, err := service.Login(ctx, "existing.user")
	if err != nil {
		t.Fatalf("expected existing user login to backfill default organization: %v", err)
	}
	assertDefaultOrganization(t, ctx, organizationRepo, organizationMemberRepo, session.User.ID, "existing-user")
}

func TestLoginUsesFallbackDefaultOrganizationPathWhenPathExists(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	userRepo := newMemoryUserRepository(identity.User{ID: 7, Username: "alice", DisplayName: "Alice"})
	tokenRepo := newMemoryUserAccessTokenRepository()
	organizationRepo := newMemoryOrganizationRepository(organizationdomain.Organization{ID: 1, Name: "Other Alice", PathKey: "alice", FullPath: "alice"})
	organizationMemberRepo := newMemoryOrganizationMemberRepository()
	service := userservice.NewServiceWithDependencies(userservice.Dependencies{
		Repo:                   userRepo,
		TokenRepo:              tokenRepo,
		OrganizationRepo:       organizationRepo,
		OrganizationMemberRepo: organizationMemberRepo,
	})

	session, err := service.Login(ctx, "alice")
	if err != nil {
		t.Fatalf("expected default organization fallback path to be created: %v", err)
	}
	assertDefaultOrganization(t, ctx, organizationRepo, organizationMemberRepo, session.User.ID, "alice-7")
}

func assertDefaultOrganization(t *testing.T, ctx context.Context, organizationRepo *memoryOrganizationRepository, memberRepo *memoryOrganizationMemberRepository, userID int64, pathKey string) {
	t.Helper()
	organizations, err := organizationRepo.List(ctx)
	if err != nil {
		t.Fatalf("expected organizations list to succeed: %v", err)
	}
	var matched organizationdomain.Organization
	for _, organization := range organizations.Values() {
		if organization.PathKey == pathKey {
			matched = organization
			break
		}
	}
	if matched.ID == 0 {
		t.Fatalf("expected default organization path %q to exist", pathKey)
	}
	member, err := memberRepo.FindByOrganizationAndUser(ctx, matched.ID, userID)
	if err != nil {
		t.Fatalf("expected default organization owner membership: %v", err)
	}
	if member.Role != "owner" {
		t.Fatalf("expected owner role, got %q", member.Role)
	}
}

type memoryUserAccessTokenRepository struct {
	tokens map[string]identity.UserAccessToken
	next   int64
}

func newMemoryUserAccessTokenRepository() *memoryUserAccessTokenRepository {
	return &memoryUserAccessTokenRepository{tokens: map[string]identity.UserAccessToken{}, next: 1}
}

func (r *memoryUserAccessTokenRepository) ListByUserID(_ context.Context, userID int64) (*collectionx.List[identity.UserAccessToken], error) {
	tokens := make([]identity.UserAccessToken, 0, len(r.tokens))
	for _, token := range r.tokens {
		if token.UserID == userID {
			tokens = append(tokens, token)
		}
	}
	slices.SortFunc(tokens, func(left, right identity.UserAccessToken) int {
		return int(left.ID - right.ID)
	})
	return collectionx.NewList(tokens...), nil
}

func (r *memoryUserAccessTokenRepository) GetByToken(_ context.Context, token string) (identity.UserAccessToken, error) {
	item, ok := r.tokens[strings.TrimSpace(token)]
	if !ok {
		return identity.UserAccessToken{}, appports.ErrNotFound
	}
	return item, nil
}

func (r *memoryUserAccessTokenRepository) Create(_ context.Context, input appports.CreateUserAccessTokenInput) (identity.UserAccessToken, error) {
	item := identity.UserAccessToken{ID: r.next, UserID: input.UserID, Name: strings.TrimSpace(input.Name), Token: strings.TrimSpace(input.Token)}
	r.next++
	r.tokens[item.Token] = item
	return item, nil
}

func (r *memoryUserAccessTokenRepository) DeleteByToken(_ context.Context, token string) error {
	token = strings.TrimSpace(token)
	if _, ok := r.tokens[token]; !ok {
		return appports.ErrNotFound
	}
	delete(r.tokens, token)
	return nil
}

type memoryOrganizationRepository struct {
	organizations map[int64]organizationdomain.Organization
	next          int64
}

func newMemoryOrganizationRepository(organizations ...organizationdomain.Organization) *memoryOrganizationRepository {
	repo := &memoryOrganizationRepository{organizations: map[int64]organizationdomain.Organization{}, next: 1}
	for _, organization := range organizations {
		if organization.ID <= 0 {
			organization.ID = repo.next
		}
		if organization.FullPath == "" {
			organization.FullPath = organization.PathKey
		}
		repo.organizations[organization.ID] = organization
		if organization.ID >= repo.next {
			repo.next = organization.ID + 1
		}
	}
	return repo
}

func (r *memoryOrganizationRepository) List(context.Context) (*collectionx.List[organizationdomain.Organization], error) {
	ids := make([]int64, 0, len(r.organizations))
	for id := range r.organizations {
		ids = append(ids, id)
	}
	slices.Sort(ids)
	organizations := make([]organizationdomain.Organization, 0, len(ids))
	for _, id := range ids {
		organizations = append(organizations, r.organizations[id])
	}
	return collectionx.NewList(organizations...), nil
}

func (r *memoryOrganizationRepository) GetByID(_ context.Context, id int64) (organizationdomain.Organization, error) {
	item, ok := r.organizations[id]
	if !ok {
		return organizationdomain.Organization{}, appports.ErrNotFound
	}
	return item, nil
}

func (r *memoryOrganizationRepository) Create(_ context.Context, input appports.CreateOrganizationInput) (organizationdomain.Organization, error) {
	pathKey := strings.TrimSpace(input.PathKey)
	for _, organization := range r.organizations {
		if organization.PathKey == pathKey {
			return organizationdomain.Organization{}, errors.New("organization path exists")
		}
	}
	item := organizationdomain.Organization{
		ID:          r.next,
		Name:        strings.TrimSpace(input.Name),
		PathKey:     pathKey,
		FullPath:    pathKey,
		Description: strings.TrimSpace(input.Description),
		Visibility:  strings.TrimSpace(input.Visibility),
	}
	r.next++
	r.organizations[item.ID] = item
	return item, nil
}

func (r *memoryOrganizationRepository) UpdateByID(_ context.Context, id int64, input appports.UpdateOrganizationInput) error {
	item, ok := r.organizations[id]
	if !ok {
		return appports.ErrNotFound
	}
	if input.Name != nil {
		item.Name = strings.TrimSpace(*input.Name)
	}
	if input.PathKey != nil {
		item.PathKey = strings.TrimSpace(*input.PathKey)
		item.FullPath = item.PathKey
	}
	if input.Description != nil {
		item.Description = strings.TrimSpace(*input.Description)
	}
	if input.Visibility != nil {
		item.Visibility = strings.TrimSpace(*input.Visibility)
	}
	r.organizations[id] = item
	return nil
}

func (r *memoryOrganizationRepository) DeleteByID(_ context.Context, id int64) error {
	if _, ok := r.organizations[id]; !ok {
		return appports.ErrNotFound
	}
	delete(r.organizations, id)
	return nil
}

type memoryOrganizationMemberRepository struct {
	members map[int64]organizationdomain.OrganizationMember
	next    int64
}

func newMemoryOrganizationMemberRepository(members ...organizationdomain.OrganizationMember) *memoryOrganizationMemberRepository {
	repo := &memoryOrganizationMemberRepository{members: map[int64]organizationdomain.OrganizationMember{}, next: 1}
	for _, member := range members {
		if member.ID <= 0 {
			member.ID = repo.next
		}
		repo.members[member.ID] = member
		if member.ID >= repo.next {
			repo.next = member.ID + 1
		}
	}
	return repo
}

func (r *memoryOrganizationMemberRepository) ListByOrganizationID(_ context.Context, organizationID int64) (*collectionx.List[organizationdomain.OrganizationMember], error) {
	members := make([]organizationdomain.OrganizationMember, 0, len(r.members))
	for _, member := range r.members {
		if member.OrganizationID == organizationID {
			members = append(members, member)
		}
	}
	slices.SortFunc(members, func(left, right organizationdomain.OrganizationMember) int {
		return int(left.ID - right.ID)
	})
	return collectionx.NewList(members...), nil
}

func (r *memoryOrganizationMemberRepository) FindByOrganizationAndUser(_ context.Context, organizationID, userID int64) (organizationdomain.OrganizationMember, error) {
	for _, member := range r.members {
		if member.OrganizationID == organizationID && member.UserID == userID {
			return member, nil
		}
	}
	return organizationdomain.OrganizationMember{}, appports.ErrNotFound
}

func (r *memoryOrganizationMemberRepository) Create(_ context.Context, input appports.CreateOrganizationMemberInput) (organizationdomain.OrganizationMember, error) {
	for _, member := range r.members {
		if member.OrganizationID == input.OrganizationID && member.UserID == input.UserID {
			return organizationdomain.OrganizationMember{}, errors.New("organization membership exists")
		}
	}
	item := organizationdomain.OrganizationMember{ID: r.next, OrganizationID: input.OrganizationID, UserID: input.UserID, Role: strings.TrimSpace(input.Role)}
	r.next++
	r.members[item.ID] = item
	return item, nil
}

var _ appports.UserAccessTokenRepository = (*memoryUserAccessTokenRepository)(nil)
var _ appports.OrganizationRepository = (*memoryOrganizationRepository)(nil)
var _ appports.OrganizationMemberRepository = (*memoryOrganizationMemberRepository)(nil)
