package wiki

import (
	"context"
	"fmt"
	"net/http"
	"strings"
	"unicode"

	"github.com/DaiYuANg/gity/internal/entity"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	projectwikipagerepo "github.com/DaiYuANg/gity/internal/repository/projectwikipage"
	userrepo "github.com/DaiYuANg/gity/internal/repository/user"
	dbxrepo "github.com/arcgolabs/dbx/repository"
	"github.com/arcgolabs/httpx"
)

type Service struct {
	projectRepo *projectrepo.Repository
	pageRepo    *projectwikipagerepo.Repository
	userRepo    *userrepo.Repository
}

type CreatePageInput struct {
	Slug         string `json:"slug"`
	Title        string `json:"title"`
	Content      string `json:"content"`
	Format       string `json:"format"`
	AuthorUserID int64  `json:"author_user_id"`
}

type UpdatePageInput struct {
	Title        *string `json:"title"`
	Content      *string `json:"content"`
	EditorUserID int64   `json:"editor_user_id"`
}

func NewService(projectRepo *projectrepo.Repository, pageRepo *projectwikipagerepo.Repository, userRepo *userrepo.Repository) *Service {
	return &Service{projectRepo: projectRepo, pageRepo: pageRepo, userRepo: userRepo}
}

func (s *Service) ListPages(ctx context.Context, projectID int64) ([]entity.ProjectWikiPage, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	items, err := s.pageRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, err
	}
	return items.Values(), nil
}

func (s *Service) GetPage(ctx context.Context, projectID int64, slug string) (entity.ProjectWikiPage, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return entity.ProjectWikiPage{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	normalizedSlug, err := normalizeSlug(slug, "")
	if err != nil {
		return entity.ProjectWikiPage{}, httpx.NewError(http.StatusBadRequest, "invalid wiki page slug", err)
	}
	return s.loadPage(ctx, projectID, normalizedSlug)
}

func (s *Service) CreatePage(ctx context.Context, projectID int64, input CreatePageInput) (entity.ProjectWikiPage, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return entity.ProjectWikiPage{}, httpx.NewError(http.StatusNotFound, "project not found", err)
	}
	if _, err := s.userRepo.GetByID(ctx, input.AuthorUserID); err != nil {
		return entity.ProjectWikiPage{}, httpx.NewError(http.StatusNotFound, "wiki author not found", err)
	}
	title := strings.TrimSpace(input.Title)
	if title == "" {
		return entity.ProjectWikiPage{}, httpx.NewError(http.StatusBadRequest, "wiki page title is required", fmt.Errorf("wiki page title is required"))
	}
	format := strings.TrimSpace(strings.ToLower(input.Format))
	if format == "" {
		format = "markdown"
	}
	if format != "markdown" {
		return entity.ProjectWikiPage{}, httpx.NewError(http.StatusBadRequest, "wiki page format must be markdown", fmt.Errorf("wiki page format must be markdown"))
	}
	slug, err := normalizeSlug(input.Slug, title)
	if err != nil {
		return entity.ProjectWikiPage{}, httpx.NewError(http.StatusBadRequest, "invalid wiki page slug", err)
	}
	if _, err := s.pageRepo.GetByProjectAndSlug(ctx, projectID, slug); err == nil {
		return entity.ProjectWikiPage{}, httpx.NewError(http.StatusConflict, "wiki page already exists", fmt.Errorf("wiki page already exists: %s", slug))
	} else if err != nil && err != dbxrepo.ErrNotFound {
		return entity.ProjectWikiPage{}, err
	}
	return s.pageRepo.Create(ctx, projectwikipagerepo.CreateInput{
		ProjectID:    projectID,
		Slug:         slug,
		Title:        title,
		Content:      input.Content,
		Format:       format,
		AuthorUserID: input.AuthorUserID,
	})
}

func (s *Service) UpdatePage(ctx context.Context, projectID int64, slug string, input UpdatePageInput) (entity.ProjectWikiPage, error) {
	page, err := s.GetPage(ctx, projectID, slug)
	if err != nil {
		return entity.ProjectWikiPage{}, err
	}
	if input.Title != nil && strings.TrimSpace(*input.Title) == "" {
		return entity.ProjectWikiPage{}, httpx.NewError(http.StatusBadRequest, "wiki page title is required", fmt.Errorf("wiki page title is required"))
	}
	if input.EditorUserID > 0 {
		if _, err := s.userRepo.GetByID(ctx, input.EditorUserID); err != nil {
			return entity.ProjectWikiPage{}, httpx.NewError(http.StatusNotFound, "wiki editor not found", err)
		}
	}
	if err := s.pageRepo.UpdateByID(ctx, page.ID, projectwikipagerepo.UpdateInput{Title: input.Title, Content: input.Content, LastEditedByUserID: input.EditorUserID}); err != nil {
		return entity.ProjectWikiPage{}, err
	}
	return s.GetPage(ctx, projectID, page.Slug)
}

func (s *Service) DeletePage(ctx context.Context, projectID int64, slug string) (entity.ProjectWikiPage, error) {
	page, err := s.GetPage(ctx, projectID, slug)
	if err != nil {
		return entity.ProjectWikiPage{}, err
	}
	if err := s.pageRepo.DeleteByID(ctx, page.ID); err != nil {
		return entity.ProjectWikiPage{}, err
	}
	return page, nil
}

func (s *Service) loadPage(ctx context.Context, projectID int64, slug string) (entity.ProjectWikiPage, error) {
	page, err := s.pageRepo.GetByProjectAndSlug(ctx, projectID, slug)
	if err != nil {
		if err == dbxrepo.ErrNotFound {
			return entity.ProjectWikiPage{}, httpx.NewError(http.StatusNotFound, "wiki page not found", err)
		}
		return entity.ProjectWikiPage{}, err
	}
	return page, nil
}

func normalizeSlug(value string, fallbackTitle string) (string, error) {
	source := strings.TrimSpace(value)
	if source == "" {
		source = strings.TrimSpace(fallbackTitle)
	}
	source = strings.ReplaceAll(source, "\\", "-")
	source = strings.ReplaceAll(source, "/", "-")
	var builder strings.Builder
	lastDash := false
	for _, r := range source {
		switch {
		case unicode.IsLetter(r) || unicode.IsDigit(r):
			builder.WriteRune(unicode.ToLower(r))
			lastDash = false
		case r == '-' || r == '_' || unicode.IsSpace(r):
			if !lastDash && builder.Len() > 0 {
				builder.WriteRune('-')
				lastDash = true
			}
		default:
			if !lastDash && builder.Len() > 0 {
				builder.WriteRune('-')
				lastDash = true
			}
		}
	}
	slug := strings.Trim(builder.String(), "-")
	if slug == "" || slug == "." || slug == ".." {
		return "", fmt.Errorf("wiki page slug is required")
	}
	if len(slug) > 160 {
		return "", fmt.Errorf("wiki page slug is too long")
	}
	return slug, nil
}
