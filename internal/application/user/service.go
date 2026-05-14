package user

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"errors"
	"fmt"
	identityports "github.com/lyonbrown4d/gity/internal/application/ports"
	identity "github.com/lyonbrown4d/gity/internal/domain/identity"
	"github.com/samber/oops"
	"log/slog"
	"strings"

	collectionx "github.com/arcgolabs/collectionx/list"
)

type Service struct {
	logger    *slog.Logger
	repo      identityports.UserRepository
	tokenRepo identityports.UserAccessTokenRepository
}

type CreateInput struct {
	Username    string `json:"username"`
	DisplayName string `json:"display_name"`
	Email       string `json:"email"`
}

type UpdateInput struct {
	Username    *string `json:"username"`
	DisplayName *string `json:"display_name"`
	Email       *string `json:"email"`
	Status      *string `json:"status"`
}

type CreateTokenInput struct {
	Name string `json:"name"`
}

type AuthSession struct {
	User         identity.User
	AccessToken  identity.UserAccessToken
	RefreshToken identity.UserAccessToken
}

func NewService(logger *slog.Logger, repo identityports.UserRepository, tokenRepo identityports.UserAccessTokenRepository) *Service {
	return &Service{logger: logger, repo: repo, tokenRepo: tokenRepo}
}

func (s *Service) List(ctx context.Context) (*collectionx.List[identity.User], error) {
	items, err := s.repo.List(ctx)
	if err != nil {
		return nil, oops.In("user").Wrapf(err, "list users")
	}
	return items, nil
}

func (s *Service) GetByID(ctx context.Context, id int64) (identity.User, error) {
	item, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return identity.User{}, oops.In("user").With("user_id", id).Wrapf(err, "load user")
	}
	return item, nil
}

func (s *Service) GetByUsername(ctx context.Context, username string) (identity.User, error) {
	item, err := s.repo.GetByUsername(ctx, username)
	if err != nil {
		return identity.User{}, oops.In("user").With("username", username).Wrapf(err, "load user by username")
	}
	return item, nil
}

func (s *Service) Create(ctx context.Context, input CreateInput) (identity.User, error) {
	if strings.TrimSpace(input.Username) == "" {
		return identity.User{}, oops.In("user").New("username is required")
	}
	if strings.TrimSpace(input.DisplayName) == "" {
		input.DisplayName = input.Username
	}
	item, err := s.repo.Create(ctx, identityports.CreateUserInput{
		Username:    input.Username,
		DisplayName: input.DisplayName,
		Email:       input.Email,
	})
	if err != nil {
		return identity.User{}, oops.In("user").With("username", input.Username, "email", input.Email).Wrapf(err, "create user")
	}
	return item, nil
}

func (s *Service) Update(ctx context.Context, id int64, input UpdateInput) (identity.User, error) {
	if input.Username != nil && strings.TrimSpace(*input.Username) == "" {
		return identity.User{}, oops.In("user").With("user_id", id).New("username is required")
	}
	if input.DisplayName != nil && strings.TrimSpace(*input.DisplayName) == "" {
		displayName := ""
		if input.Username != nil {
			displayName = strings.TrimSpace(*input.Username)
		}
		input.DisplayName = &displayName
	}
	if err := s.repo.UpdateByID(ctx, id, identityports.UpdateUserInput{
		Username:    input.Username,
		DisplayName: input.DisplayName,
		Email:       input.Email,
	}); err != nil {
		return identity.User{}, oops.In("user").With("user_id", id).Wrapf(err, "update user")
	}
	item, err := s.repo.GetByID(ctx, id)
	if err != nil {
		return identity.User{}, oops.In("user").With("user_id", id).Wrapf(err, "reload user")
	}
	return item, nil
}

func (s *Service) Delete(ctx context.Context, id int64) error {
	if id <= 0 {
		return oops.In("user").With("user_id", id).New("user id is required")
	}
	if err := s.repo.DeleteByID(ctx, id); err != nil {
		return oops.In("user").With("user_id", id).Wrapf(err, "delete user")
	}
	return nil
}

func (s *Service) Login(ctx context.Context, username string) (AuthSession, error) {
	username = strings.TrimSpace(username)
	if username == "" {
		return AuthSession{}, oops.In("user").New("username is required")
	}
	user, err := s.repo.GetByUsername(ctx, username)
	if err != nil {
		if !errors.Is(err, identityports.ErrNotFound) {
			return AuthSession{}, oops.In("user").With("username", username).Wrapf(err, "load login user")
		}
		user, err = s.Create(ctx, CreateInput{
			Username:    username,
			DisplayName: username,
			Email:       username + "@local.gity",
		})
		if err != nil {
			return AuthSession{}, oops.In("user").With("username", username).Wrapf(err, "create login user")
		}
	}
	return s.createSession(ctx, user)
}

func (s *Service) Refresh(ctx context.Context, refreshToken string) (AuthSession, error) {
	record, err := s.tokenRepo.GetByToken(ctx, strings.TrimSpace(refreshToken))
	if err != nil {
		return AuthSession{}, oops.In("user").Wrapf(err, "load refresh token")
	}
	user, err := s.repo.GetByID(ctx, record.UserID)
	if err != nil {
		return AuthSession{}, oops.In("user").With("user_id", record.UserID).Wrapf(err, "load refresh user")
	}
	if err := s.tokenRepo.DeleteByToken(ctx, record.Token); err != nil && !errors.Is(err, identityports.ErrNotFound) {
		return AuthSession{}, oops.In("user").With("user_id", record.UserID).Wrapf(err, "revoke refresh token")
	}
	return s.createSession(ctx, user)
}

func (s *Service) RevokeToken(ctx context.Context, token string) error {
	token = strings.TrimSpace(token)
	if token == "" {
		return nil
	}
	if err := s.tokenRepo.DeleteByToken(ctx, token); err != nil && !errors.Is(err, identityports.ErrNotFound) {
		return oops.In("user").Wrapf(err, "revoke token")
	}
	return nil
}

func (s *Service) GetByToken(ctx context.Context, token string) (identity.User, error) {
	record, err := s.tokenRepo.GetByToken(ctx, strings.TrimSpace(token))
	if err != nil {
		return identity.User{}, oops.In("user").Wrapf(err, "load access token")
	}
	item, err := s.repo.GetByID(ctx, record.UserID)
	if err != nil {
		return identity.User{}, oops.In("user").With("user_id", record.UserID).Wrapf(err, "load token user")
	}
	return item, nil
}

func (s *Service) ListTokens(ctx context.Context, userID int64) ([]identity.UserAccessToken, error) {
	if _, err := s.repo.GetByID(ctx, userID); err != nil {
		return nil, oops.In("user").With("user_id", userID).Wrapf(err, "load user")
	}
	items, err := s.tokenRepo.ListByUserID(ctx, userID)
	if err != nil {
		return nil, oops.In("user").With("user_id", userID).Wrapf(err, "list user access tokens")
	}
	return items.Values(), nil
}

func (s *Service) CreateToken(ctx context.Context, userID int64, input CreateTokenInput) (identity.UserAccessToken, error) {
	if _, err := s.repo.GetByID(ctx, userID); err != nil {
		return identity.UserAccessToken{}, oops.In("user").With("user_id", userID).Wrapf(err, "load user")
	}
	name := strings.TrimSpace(input.Name)
	if name == "" {
		name = "default"
	}
	token, err := generateToken()
	if err != nil {
		return identity.UserAccessToken{}, oops.In("user").With("user_id", userID).Wrapf(err, "generate access token")
	}
	record, err := s.tokenRepo.Create(ctx, identityports.CreateUserAccessTokenInput{
		UserID: userID,
		Name:   name,
		Token:  token,
	})
	if err != nil {
		return identity.UserAccessToken{}, oops.In("user").With("user_id", userID, "token_name", name).Wrapf(err, "create user access token")
	}
	return record, nil
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

func generateToken() (string, error) {
	buf := make([]byte, 24)
	if _, err := rand.Read(buf); err != nil {
		return "", fmt.Errorf("generate token: %w", err)
	}
	return "gity_" + hex.EncodeToString(buf), nil
}
