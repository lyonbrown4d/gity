package user_test

import (
	"context"
	"testing"

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
	assertDefaultOrganization(ctx, t, organizationRepo, organizationMemberRepo, session.User.ID, "alice-owner")

	if _, loginErr := service.Login(ctx, "Alice Owner"); loginErr != nil {
		t.Fatalf("expected repeated login to remain idempotent: %v", loginErr)
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
	assertDefaultOrganization(ctx, t, organizationRepo, organizationMemberRepo, session.User.ID, "existing-user")
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
	assertDefaultOrganization(ctx, t, organizationRepo, organizationMemberRepo, session.User.ID, "alice-7")
}

func assertDefaultOrganization(ctx context.Context, t *testing.T, organizationRepo *memoryOrganizationRepository, memberRepo *memoryOrganizationMemberRepository, userID int64, pathKey string) {
	t.Helper()
	organizations, err := organizationRepo.List(ctx)
	if err != nil {
		t.Fatalf("expected organizations list to succeed: %v", err)
	}
	var matched organizationdomain.Organization
	values := organizations.Values()
	for index := range values {
		if values[index].PathKey == pathKey {
			matched = values[index]
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
