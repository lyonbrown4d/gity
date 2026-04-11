package user

import (
	"context"
	"fmt"
	"log/slog"
	"strings"

	"github.com/DaiYuANg/arcgo/collectionx"
	"github.com/DaiYuANg/gity/internal/entity"
	userrepo "github.com/DaiYuANg/gity/internal/repository/user"
)

type Service struct {
	logger *slog.Logger
	repo   *userrepo.Repository
}

type CreateInput struct {
	Username    string `json:"username"`
	DisplayName string `json:"display_name"`
	Email       string `json:"email"`
}

func NewService(logger *slog.Logger, repo *userrepo.Repository) *Service {
	return &Service{logger: logger, repo: repo}
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
