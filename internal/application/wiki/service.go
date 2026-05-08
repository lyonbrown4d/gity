package wiki

import (
	"context"
	"errors"
	apperror "github.com/DaiYuANg/gity/internal/application/app_error"
	appports "github.com/DaiYuANg/gity/internal/application/ports"
	wikidomain "github.com/DaiYuANg/gity/internal/domain/wiki"
	"github.com/samber/oops"
	"strings"
	"unicode"
)

type Service struct {
	projectRepo appports.ProjectRepository
	pageRepo    appports.ProjectWikiPageRepository
	userRepo    appports.UserRepository
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

func NewService(projectRepo appports.ProjectRepository, pageRepo appports.ProjectWikiPageRepository, userRepo appports.UserRepository) *Service {
	return &Service{projectRepo: projectRepo, pageRepo: pageRepo, userRepo: userRepo}
}

func (s *Service) ListPages(ctx context.Context, projectID int64) ([]wikidomain.ProjectWikiPage, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return nil, apperror.NotFound("project not found", err)
	}
	items, err := s.pageRepo.ListByProjectID(ctx, projectID)
	if err != nil {
		return nil, err
	}
	return items.Values(), nil
}

func (s *Service) GetPage(ctx context.Context, projectID int64, slug string) (wikidomain.ProjectWikiPage, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return wikidomain.ProjectWikiPage{}, apperror.NotFound("project not found", err)
	}
	normalizedSlug, err := normalizeSlug(slug, "")
	if err != nil {
		return wikidomain.ProjectWikiPage{}, apperror.BadRequest("invalid wiki page slug", err)
	}
	return s.loadPage(ctx, projectID, normalizedSlug)
}

func (s *Service) CreatePage(ctx context.Context, projectID int64, input CreatePageInput) (wikidomain.ProjectWikiPage, error) {
	if _, err := s.projectRepo.GetByID(ctx, projectID); err != nil {
		return wikidomain.ProjectWikiPage{}, apperror.NotFound("project not found", err)
	}
	if _, err := s.userRepo.GetByID(ctx, input.AuthorUserID); err != nil {
		return wikidomain.ProjectWikiPage{}, apperror.NotFound("wiki author not found", err)
	}
	title := strings.TrimSpace(input.Title)
	if title == "" {
		return wikidomain.ProjectWikiPage{}, apperror.BadRequest("wiki page title is required", oops.In("wiki").With("project_id", projectID, "author_user_id", input.AuthorUserID).New("wiki page title is required"))
	}
	format := strings.TrimSpace(strings.ToLower(input.Format))
	if format == "" {
		format = "markdown"
	}
	if format != "markdown" {
		return wikidomain.ProjectWikiPage{}, apperror.BadRequest("wiki page format must be markdown", oops.In("wiki").With("project_id", projectID, "format", format).New("wiki page format must be markdown"))
	}
	slug, err := normalizeSlug(input.Slug, title)
	if err != nil {
		return wikidomain.ProjectWikiPage{}, apperror.BadRequest("invalid wiki page slug", err)
	}
	if _, err := s.pageRepo.GetByProjectAndSlug(ctx, projectID, slug); err == nil {
		return wikidomain.ProjectWikiPage{}, apperror.Conflict("wiki page already exists", oops.In("wiki").With("project_id", projectID, "slug", slug).New("wiki page already exists"))
	} else if !errors.Is(err, appports.ErrNotFound) {
		return wikidomain.ProjectWikiPage{}, err
	}
	return s.pageRepo.Create(ctx, appports.CreateProjectWikiPageInput{
		ProjectID:    projectID,
		Slug:         slug,
		Title:        title,
		Content:      input.Content,
		Format:       format,
		AuthorUserID: input.AuthorUserID,
	})
}

func (s *Service) UpdatePage(ctx context.Context, projectID int64, slug string, input UpdatePageInput) (wikidomain.ProjectWikiPage, error) {
	page, err := s.GetPage(ctx, projectID, slug)
	if err != nil {
		return wikidomain.ProjectWikiPage{}, err
	}
	if input.Title != nil && strings.TrimSpace(*input.Title) == "" {
		return wikidomain.ProjectWikiPage{}, apperror.BadRequest("wiki page title is required", oops.In("wiki").With("project_id", projectID, "slug", slug).New("wiki page title is required"))
	}
	if input.EditorUserID > 0 {
		if _, err := s.userRepo.GetByID(ctx, input.EditorUserID); err != nil {
			return wikidomain.ProjectWikiPage{}, apperror.NotFound("wiki editor not found", err)
		}
	}
	if err := s.pageRepo.UpdateByID(ctx, page.ID, appports.UpdateProjectWikiPageInput{Title: input.Title, Content: input.Content, LastEditedByUserID: input.EditorUserID}); err != nil {
		return wikidomain.ProjectWikiPage{}, err
	}
	return s.GetPage(ctx, projectID, page.Slug)
}

func (s *Service) DeletePage(ctx context.Context, projectID int64, slug string) (wikidomain.ProjectWikiPage, error) {
	page, err := s.GetPage(ctx, projectID, slug)
	if err != nil {
		return wikidomain.ProjectWikiPage{}, err
	}
	if err := s.pageRepo.DeleteByID(ctx, page.ID); err != nil {
		return wikidomain.ProjectWikiPage{}, err
	}
	return page, nil
}

func (s *Service) loadPage(ctx context.Context, projectID int64, slug string) (wikidomain.ProjectWikiPage, error) {
	page, err := s.pageRepo.GetByProjectAndSlug(ctx, projectID, slug)
	if err != nil {
		if errors.Is(err, appports.ErrNotFound) {
			return wikidomain.ProjectWikiPage{}, apperror.NotFound("wiki page not found", err)
		}
		return wikidomain.ProjectWikiPage{}, err
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
			if err := appendSlugRune(&builder, unicode.ToLower(r)); err != nil {
				return "", err
			}
			lastDash = false
		case r == '-' || r == '_' || unicode.IsSpace(r):
			if !lastDash && builder.Len() > 0 {
				if err := appendSlugRune(&builder, '-'); err != nil {
					return "", err
				}
				lastDash = true
			}
		default:
			if !lastDash && builder.Len() > 0 {
				if err := appendSlugRune(&builder, '-'); err != nil {
					return "", err
				}
				lastDash = true
			}
		}
	}
	slug := strings.Trim(builder.String(), "-")
	if slug == "" || slug == "." || slug == ".." {
		return "", oops.In("wiki").With("value", value, "fallback_title", fallbackTitle).New("wiki page slug is required")
	}
	if len(slug) > 160 {
		return "", oops.In("wiki").With("slug", slug, "length", len(slug)).New("wiki page slug is too long")
	}
	return slug, nil
}

func appendSlugRune(builder *strings.Builder, value rune) error {
	if _, err := builder.WriteRune(value); err != nil {
		return oops.In("wiki").With("rune", string(value)).Wrapf(err, "append wiki slug rune")
	}
	return nil
}
