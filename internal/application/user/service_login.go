package user

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"unicode"

	appports "github.com/lyonbrown4d/gity/internal/application/ports"
	identity "github.com/lyonbrown4d/gity/internal/domain/identity"
	"github.com/samber/oops"
)

func (s *Service) Login(ctx context.Context, username string) (AuthSession, error) {
	username = strings.TrimSpace(username)
	if username == "" {
		return AuthSession{}, oops.In("user").New("username is required")
	}
	user, err := s.repo.GetByUsername(ctx, username)
	if err != nil {
		if !errors.Is(err, appports.ErrNotFound) {
			return AuthSession{}, oops.In("user").With("username", username).Wrapf(err, "load login user")
		}
		firstUser, firstUserErr := s.isFirstUser(ctx)
		if firstUserErr != nil {
			return AuthSession{}, firstUserErr
		}
		user, err = s.Create(ctx, CreateInput{
			Username:     username,
			DisplayName:  username,
			Email:        username + "@local.gity",
			IsSuperAdmin: firstUser,
		})
		if err != nil {
			return AuthSession{}, oops.In("user").With("username", username).Wrapf(err, "create login user")
		}
	}
	if err := s.ensureDefaultOrganization(ctx, user); err != nil {
		return AuthSession{}, err
	}
	return s.createSession(ctx, user)
}

func (s *Service) ensureDefaultOrganization(ctx context.Context, user identity.User) error {
	if s.organizationRepo == nil || s.organizationMemberRepo == nil || user.ID <= 0 {
		return nil
	}
	hasMembership, err := s.hasOrganizationMembership(ctx, user.ID)
	if err != nil {
		return err
	}
	if hasMembership {
		return nil
	}
	pathKey := defaultOrganizationPathKey(user)
	item, err := s.organizationRepo.Create(ctx, appports.CreateOrganizationInput{
		Name:        user.DisplayName,
		PathKey:     pathKey,
		Description: "Personal project owner namespace for " + user.Username + ".",
		Visibility:  "private",
	})
	if err != nil {
		fallbackPathKey := fmt.Sprintf("%s-%d", pathKey, user.ID)
		item, err = s.organizationRepo.Create(ctx, appports.CreateOrganizationInput{
			Name:        user.DisplayName,
			PathKey:     fallbackPathKey,
			Description: "Personal project owner namespace for " + user.Username + ".",
			Visibility:  "private",
		})
		if err != nil {
			return oops.In("user").With("user_id", user.ID, "username", user.Username).Wrapf(err, "create default organization")
		}
	}
	if _, err := s.organizationMemberRepo.Create(ctx, appports.CreateOrganizationMemberInput{OrganizationID: item.ID, UserID: user.ID, Role: "owner"}); err != nil {
		return oops.In("user").With("user_id", user.ID, "organization_id", item.ID).Wrapf(err, "create default organization owner membership")
	}
	return nil
}

func (s *Service) hasOrganizationMembership(ctx context.Context, userID int64) (bool, error) {
	organizations, err := s.organizationRepo.List(ctx)
	if err != nil {
		return false, oops.In("user").With("user_id", userID).Wrapf(err, "list organizations before default owner bootstrap")
	}
	values := organizations.Values()
	for index := range values {
		organizationID := values[index].ID
		if _, err := s.organizationMemberRepo.FindByOrganizationAndUser(ctx, organizationID, userID); err == nil {
			return true, nil
		} else if !errors.Is(err, appports.ErrNotFound) {
			return false, oops.In("user").With("user_id", userID, "organization_id", organizationID).Wrapf(err, "check default organization membership")
		}
	}
	return false, nil
}

func defaultOrganizationPathKey(user identity.User) string {
	source := strings.TrimSpace(strings.ToLower(user.Username))
	pathRunes := make([]rune, 0, len(source))
	lastDash := false
	for _, value := range source {
		if unicode.IsLetter(value) || unicode.IsDigit(value) {
			pathRunes = append(pathRunes, value)
			lastDash = false
			continue
		}
		if !lastDash {
			pathRunes = append(pathRunes, '-')
			lastDash = true
		}
	}
	pathKey := strings.Trim(string(pathRunes), "-")
	if pathKey == "" {
		return fmt.Sprintf("user-%d", user.ID)
	}
	return pathKey
}

func (s *Service) isFirstUser(ctx context.Context) (bool, error) {
	users, err := s.repo.List(ctx)
	if err != nil {
		return false, oops.In("user").Wrapf(err, "check bootstrap user")
	}
	return users.Len() == 0, nil
}

func (s *Service) createSession(ctx context.Context, user identity.User) (AuthSession, error) {
	accessToken, err := s.CreateToken(ctx, user.ID, CreateTokenInput{Name: "access"})
	if err != nil {
		return AuthSession{}, err
	}
	refreshToken, err := s.CreateToken(ctx, user.ID, CreateTokenInput{Name: "refresh"})
	if err != nil {
		return AuthSession{}, err
	}
	return AuthSession{User: user, AccessToken: accessToken, RefreshToken: refreshToken}, nil
}
