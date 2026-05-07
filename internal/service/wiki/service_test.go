package wiki_test

import (
	"context"
	"fmt"
	"path/filepath"
	"strings"
	"testing"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
	"github.com/DaiYuANg/gity/internal/repository/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/repository/namespace"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	projectwikipagerepo "github.com/DaiYuANg/gity/internal/repository/projectwikipage"
	userrepo "github.com/DaiYuANg/gity/internal/repository/user"
	wikiservice "github.com/DaiYuANg/gity/internal/service/wiki"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestProjectWikiPageFlow(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	db := openTestDB(t)
	defer db.Close()
	if err := core.EnsureSchema(ctx, db); err != nil {
		t.Fatalf("ensure schema: %v", err)
	}

	namespaceRepository, err := namespacerepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new namespace repo: %v", err)
	}
	projectRepository, err := projectrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new project repo: %v", err)
	}
	userRepository, err := userrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new user repo: %v", err)
	}
	pageRepository, err := projectwikipagerepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new wiki page repo: %v", err)
	}
	service := wikiservice.NewService(projectRepository, pageRepository, userRepository)

	author, err := userRepository.Create(ctx, userrepo.CreateInput{
		Username:    "alice",
		DisplayName: "Alice",
		Email:       "alice@gity.dev",
	})
	if err != nil {
		t.Fatalf("create user: %v", err)
	}
	namespace, err := namespaceRepository.Create(ctx, namespacerepo.CreateInput{
		Kind:    "group",
		Name:    "Core Team",
		PathKey: "core-team",
	})
	if err != nil {
		t.Fatalf("create namespace: %v", err)
	}
	project, err := projectRepository.Create(ctx, projectrepo.CreateInput{
		NamespaceID: namespace.ID,
		Name:        "Gity",
		PathKey:     "gity",
		Visibility:  "private",
	}, namespace)
	if err != nil {
		t.Fatalf("create project: %v", err)
	}

	created, err := service.CreatePage(ctx, project.ID, wikiservice.CreatePageInput{
		Title:        "Getting Started",
		Content:      "# Getting Started\n",
		AuthorUserID: author.ID,
	})
	if err != nil {
		t.Fatalf("create wiki page: %v", err)
	}
	if created.Slug != "getting-started" || created.Format != "markdown" || created.AuthorUserID != author.ID {
		t.Fatalf("unexpected created wiki page: %+v", created)
	}

	if _, err := service.CreatePage(ctx, project.ID, wikiservice.CreatePageInput{
		Slug:         "getting-started",
		Title:        "Duplicate",
		Content:      "duplicate",
		AuthorUserID: author.ID,
	}); err == nil {
		t.Fatalf("expected duplicate wiki page slug to fail")
	}

	pages, err := service.ListPages(ctx, project.ID)
	if err != nil {
		t.Fatalf("list wiki pages: %v", err)
	}
	if len(pages) != 1 || pages[0].ID != created.ID {
		t.Fatalf("unexpected wiki page list: %+v", pages)
	}

	loaded, err := service.GetPage(ctx, project.ID, "getting-started")
	if err != nil {
		t.Fatalf("get wiki page: %v", err)
	}
	if loaded.ID != created.ID || !strings.Contains(loaded.Content, "Getting Started") {
		t.Fatalf("unexpected loaded wiki page: %+v", loaded)
	}

	updatedTitle := "Getting Started v2"
	updatedContent := "# Getting Started v2\n\nUpdated."
	updated, err := service.UpdatePage(ctx, project.ID, "getting-started", wikiservice.UpdatePageInput{
		Title:        &updatedTitle,
		Content:      &updatedContent,
		EditorUserID: author.ID,
	})
	if err != nil {
		t.Fatalf("update wiki page: %v", err)
	}
	if updated.Title != updatedTitle || updated.Content != updatedContent || updated.LastEditedByUserID != author.ID {
		t.Fatalf("unexpected updated wiki page: %+v", updated)
	}

	deleted, err := service.DeletePage(ctx, project.ID, "getting-started")
	if err != nil {
		t.Fatalf("delete wiki page: %v", err)
	}
	if deleted.ID != created.ID {
		t.Fatalf("unexpected deleted wiki page: %+v", deleted)
	}
	if _, err := service.GetPage(ctx, project.ID, "getting-started"); err == nil {
		t.Fatalf("expected deleted wiki page to be missing")
	}
}

func openTestDB(t *testing.T) *dbx.DB {
	t.Helper()
	dbPath := filepath.Join(t.TempDir(), "gity-test.db")
	db, err := dbx.Open(
		dbx.WithDriver("sqlite"),
		dbx.WithDSN(fmt.Sprintf("file:%s?_pragma=foreign_keys(1)", dbPath)),
		dbx.WithDialect(sqliteDialect.New()),
	)
	if err != nil {
		t.Fatalf("open db: %v", err)
	}
	return db
}
