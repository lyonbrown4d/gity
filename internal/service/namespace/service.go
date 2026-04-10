package namespace

import (
	"context"
	"fmt"
	"log/slog"
	"strings"

	"github.com/DaiYuANg/arcgo/collectionx"
	"github.com/DaiYuANg/gity/internal/entity"
	namespacerepo "github.com/DaiYuANg/gity/internal/repository/namespace"
)

type Service struct {
	logger *slog.Logger
	repo   *namespacerepo.Repository
}

type CreateInput struct {
	Kind        string `json:"kind"`
	Name        string `json:"name"`
	PathKey     string `json:"path_key"`
	Description string `json:"description"`
}

func NewService(logger *slog.Logger, repo *namespacerepo.Repository) *Service {
	return &Service{logger: logger, repo: repo}
}

func (s *Service) List(ctx context.Context) (collectionx.List[entity.Namespace], error) {
	return s.repo.List(ctx)
}

func (s *Service) GetByID(ctx context.Context, id int64) (entity.Namespace, error) {
	return s.repo.GetByID(ctx, id)
}

func (s *Service) Create(ctx context.Context, input CreateInput) (entity.Namespace, error) {
	if strings.TrimSpace(input.Name) == "" {
		return entity.Namespace{}, fmt.Errorf("namespace name is required")
	}
	if strings.TrimSpace(input.PathKey) == "" {
		return entity.Namespace{}, fmt.Errorf("namespace path_key is required")
	}
	if strings.TrimSpace(input.Kind) == "" {
		input.Kind = "group"
	}
	return s.repo.Create(ctx, namespacerepo.CreateInput{
		Kind:        input.Kind,
		Name:        input.Name,
		PathKey:     input.PathKey,
		Description: input.Description,
	})
}
