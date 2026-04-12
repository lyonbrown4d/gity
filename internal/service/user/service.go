package user

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"log/slog"
	"strings"

	"github.com/DaiYuANg/arcgo/collectionx"
	"github.com/DaiYuANg/gity/internal/entity"
	userrepo "github.com/DaiYuANg/gity/internal/repository/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/repository/usertoken"
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

type CreateTokenInput struct {
	Name string `json:"name"`
}

func NewService(logger *slog.Logger, repo *userrepo.Repository, tokenRepo *usertokenrepo.Repository) *Service {
	return &Service{logger: logger, repo: repo, tokenRepo: tokenRepo}
}

func (s *Service) List(ctx context.Context) (collectionx.List[entity.User], error) {
	return s.repo.List(ctx)
}

func (s *Service) GetByID(ctx context.Context, id int64) (entity.User, error) {
	return s.repo.GetByID(ctx, id)
}

func (s *Service) Create(ctx context.Context, input CreateInput) (entity.User, error) {
	if strings.TrimSpace(input.Username) == "" {
		return entity.User{}, fmt.Errorf("username is required")
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

func (s *Service) ListTokens(ctx context.Context, userID int64) ([]entity.UserAccessToken, error) {
	if _, err := s.repo.GetByID(ctx, userID); err != nil {
		return nil, fmt.Errorf("load user: %w", err)
	}
	items, err := s.tokenRepo.ListByUserID(ctx, userID)
	if err != nil {
		return nil, err
	}
	return items.Values(), nil
}

func (s *Service) CreateToken(ctx context.Context, userID int64, input CreateTokenInput) (entity.UserAccessToken, error) {
	if _, err := s.repo.GetByID(ctx, userID); err != nil {
		return entity.UserAccessToken{}, fmt.Errorf("load user: %w", err)
	}
	name := strings.TrimSpace(input.Name)
	if name == "" {
		name = "default"
	}
	token, err := generateToken()
	if err != nil {
		return entity.UserAccessToken{}, err
	}
	return s.tokenRepo.Create(ctx, usertokenrepo.CreateInput{
		UserID: userID,
		Name:   name,
		Token:  token,
	})
}

func generateToken() (string, error) {
	buf := make([]byte, 24)
	if _, err := rand.Read(buf); err != nil {
		return "", fmt.Errorf("generate token: %w", err)
	}
	return "gity_" + hex.EncodeToString(buf), nil
}
