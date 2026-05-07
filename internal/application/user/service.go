package user

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"errors"
	"fmt"
	identity "github.com/DaiYuANg/gity/internal/domain/identity"
	"log/slog"
	"strings"

	userrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/usertoken"
	collectionx "github.com/arcgolabs/collectionx/list"
	dbxrepo "github.com/arcgolabs/dbx/repository"
)

type Service struct {
	logger    *slog.Logger
	repo      *userrepo.Repository
	tokenRepo *usertokenrepo.Repository
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

func NewService(logger *slog.Logger, repo *userrepo.Repository, tokenRepo *usertokenrepo.Repository) *Service {
	return &Service{logger: logger, repo: repo, tokenRepo: tokenRepo}
}

func (s *Service) List(ctx context.Context) (*collectionx.List[identity.User], error) {
	return s.repo.List(ctx)
}

func (s *Service) GetByID(ctx context.Context, id int64) (identity.User, error) {
	return s.repo.GetByID(ctx, id)
}

func (s *Service) GetByUsername(ctx context.Context, username string) (identity.User, error) {
	return s.repo.GetByUsername(ctx, username)
}

func (s *Service) Create(ctx context.Context, input CreateInput) (identity.User, error) {
	if strings.TrimSpace(input.Username) == "" {
		return identity.User{}, fmt.Errorf("username is required")
	}
	if strings.TrimSpace(input.DisplayName) == "" {
		input.DisplayName = input.Username
	}
	return s.repo.Create(ctx, userrepo.CreateInput{
		Username:    input.Username,
		DisplayName: input.DisplayName,
		Email:       input.Email,
	})
}

func (s *Service) Update(ctx context.Context, id int64, input UpdateInput) (identity.User, error) {
	if input.Username != nil && strings.TrimSpace(*input.Username) == "" {
		return identity.User{}, fmt.Errorf("username is required")
	}
	if input.DisplayName != nil && strings.TrimSpace(*input.DisplayName) == "" {
		displayName := ""
		if input.Username != nil {
			displayName = strings.TrimSpace(*input.Username)
		}
		input.DisplayName = &displayName
	}
	if err := s.repo.UpdateByID(ctx, id, userrepo.UpdateInput{
		Username:    input.Username,
		DisplayName: input.DisplayName,
		Email:       input.Email,
	}); err != nil {
		return identity.User{}, err
	}
	return s.repo.GetByID(ctx, id)
}

func (s *Service) Delete(ctx context.Context, id int64) error {
	if id <= 0 {
		return fmt.Errorf("user id is required")
	}
	return s.repo.DeleteByID(ctx, id)
}

func (s *Service) Login(ctx context.Context, username string) (AuthSession, error) {
	username = strings.TrimSpace(username)
	if username == "" {
		return AuthSession{}, fmt.Errorf("username is required")
	}
	user, err := s.repo.GetByUsername(ctx, username)
	if err != nil {
		if !errors.Is(err, dbxrepo.ErrNotFound) {
			return AuthSession{}, err
		}
		user, err = s.Create(ctx, CreateInput{
			Username:    username,
			DisplayName: username,
			Email:       username + "@local.gity",
		})
		if err != nil {
			return AuthSession{}, err
		}
	}
	return s.createSession(ctx, user)
}

func (s *Service) Refresh(ctx context.Context, refreshToken string) (AuthSession, error) {
	record, err := s.tokenRepo.GetByToken(ctx, strings.TrimSpace(refreshToken))
	if err != nil {
		return AuthSession{}, err
	}
	user, err := s.repo.GetByID(ctx, record.UserID)
	if err != nil {
		return AuthSession{}, err
	}
	_ = s.tokenRepo.DeleteByToken(ctx, record.Token)
	return s.createSession(ctx, user)
}

func (s *Service) RevokeToken(ctx context.Context, token string) error {
	token = strings.TrimSpace(token)
	if token == "" {
		return nil
	}
	if err := s.tokenRepo.DeleteByToken(ctx, token); err != nil && !errors.Is(err, dbxrepo.ErrNotFound) {
		return err
	}
	return nil
}

func (s *Service) GetByToken(ctx context.Context, token string) (identity.User, error) {
	record, err := s.tokenRepo.GetByToken(ctx, strings.TrimSpace(token))
	if err != nil {
		return identity.User{}, err
	}
	return s.repo.GetByID(ctx, record.UserID)
}

func (s *Service) ListTokens(ctx context.Context, userID int64) ([]identity.UserAccessToken, error) {
	if _, err := s.repo.GetByID(ctx, userID); err != nil {
		return nil, fmt.Errorf("load user: %w", err)
	}
	items, err := s.tokenRepo.ListByUserID(ctx, userID)
	if err != nil {
		return nil, err
	}
	return items.Values(), nil
}

func (s *Service) CreateToken(ctx context.Context, userID int64, input CreateTokenInput) (identity.UserAccessToken, error) {
	if _, err := s.repo.GetByID(ctx, userID); err != nil {
		return identity.UserAccessToken{}, fmt.Errorf("load user: %w", err)
	}
	name := strings.TrimSpace(input.Name)
	if name == "" {
		name = "default"
	}
	token, err := generateToken()
	if err != nil {
		return identity.UserAccessToken{}, err
	}
	return s.tokenRepo.Create(ctx, usertokenrepo.CreateInput{
		UserID: userID,
		Name:   name,
		Token:  token,
	})
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
