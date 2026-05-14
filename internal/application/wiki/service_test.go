package wiki_test

import (
	"context"
	"fmt"
	"path/filepath"
	"strings"
	"testing"

	wikiservice "github.com/lyonbrown4d/gity/internal/application/wiki"
	"github.com/lyonbrown4d/gity/internal/infrastructure/persistence/core"
	organizationrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/organization"
	projectrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project"
	projectwikipagerepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_wiki_page"
	userrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/user"
	"github.com/lyonbrown4d/gity/internal/testutil"

	"github.com/arcgolabs/dbx"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestProjectWikiPageFlow(t *testing.T) {
	t.Parallel()

	fixture := newWikiFixture(t)
	pageID := assertCreateWikiPage(t, fixture)
	otherPageID := assertCreateAnotherWikiPage(t, fixture)
	assertDuplicateWikiPageFails(t, fixture)
	assertListWikiPages(t, fixture, pageID, otherPageID)
	assertGetWikiPage(t, fixture, pageID)
	assertUpdateWikiPage(t, fixture)
	assertDeleteWikiPage(t, fixture, pageID)
}

type wikiFixture struct {
	ctx       context.Context
	projectID int64
	authorID  int64
	service   *wikiservice.Service
}

func newWikiFixture(t *testing.T) wikiFixture {
	t.Helper()

	ctx := context.Background()
	db := openTestDB(t)
	testutil.CleanupClose(t, "db", db)
	testutil.RequireNoError(t, core.EnsureSchema(ctx, db), "ensure schema")

	organizationRepository := testutil.Must(organizationrepo.NewRepository(db))
	projectRepository := testutil.Must(projectrepo.NewRepository(db))
	userRepository := testutil.Must(userrepo.NewRepository(db))
	pageRepository := testutil.Must(projectwikipagerepo.NewRepository(db))
	service := wikiservice.NewService(projectRepository, pageRepository, userRepository)

	author := testutil.Must(userRepository.Create(ctx, userrepo.CreateInput{
		Username:    "alice",
		DisplayName: "Alice",
		Email:       "alice@gity.dev",
	}))
	organization := testutil.Must(organizationRepository.Create(ctx, organizationrepo.CreateInput{
		Name:    "Core Team",
		PathKey: "core-team",
	}))
	project := testutil.Must(projectRepository.Create(ctx, projectrepo.CreateInput{
		OrganizationID: organization.ID,
		Name:           "Gity",
		PathKey:        "gity",
		Visibility:     "private",
	}, organization))

	return wikiFixture{
		ctx:       ctx,
		projectID: project.ID,
		authorID:  author.ID,
		service:   service,
	}
}

func assertCreateWikiPage(t *testing.T, fixture wikiFixture) int64 {
	t.Helper()

	created := testutil.Must(fixture.service.CreatePage(fixture.ctx, fixture.projectID, wikiservice.CreatePageInput{
		Title:        "Getting Started",
		Content:      "# Getting Started\n",
		AuthorUserID: fixture.authorID,
	}))
	if created.Slug != "getting-started" || created.Format != "markdown" || created.AuthorUserID != fixture.authorID {
		t.Fatalf("unexpected created wiki page: %+v", created)
	}
	return created.ID
}

func assertCreateAnotherWikiPage(t *testing.T, fixture wikiFixture) int64 {
	t.Helper()

	created := testutil.Must(fixture.service.CreatePage(fixture.ctx, fixture.projectID, wikiservice.CreatePageInput{
		Slug:         "architecture-notes",
		Title:        "Architecture Notes",
		Content:      "# Architecture Notes\n",
		AuthorUserID: fixture.authorID,
	}))
	if created.Slug != "architecture-notes" || created.ProjectID != fixture.projectID {
		t.Fatalf("unexpected second wiki page: %+v", created)
	}
	return created.ID
}

func assertDuplicateWikiPageFails(t *testing.T, fixture wikiFixture) {
	t.Helper()

	if _, createPageErr := fixture.service.CreatePage(fixture.ctx, fixture.projectID, wikiservice.CreatePageInput{
		Slug:         "getting-started",
		Title:        "Duplicate",
		Content:      "duplicate",
		AuthorUserID: fixture.authorID,
	}); createPageErr == nil {
		t.Fatalf("expected duplicate wiki page slug to fail")
	}
}

func assertListWikiPages(t *testing.T, fixture wikiFixture, pageID, otherPageID int64) {
	t.Helper()

	pages := testutil.Must(fixture.service.ListPages(fixture.ctx, fixture.projectID))
	if len(pages) != 2 {
		t.Fatalf("unexpected wiki page list: %+v", pages)
	}
	seen := map[int64]bool{}
	for index := range pages {
		page := &pages[index]
		seen[page.ID] = true
	}
	if !seen[pageID] || !seen[otherPageID] {
		t.Fatalf("wiki page list does not include expected pages: %+v", pages)
	}
}

func assertGetWikiPage(t *testing.T, fixture wikiFixture, pageID int64) {
	t.Helper()

	loaded := testutil.Must(fixture.service.GetPage(fixture.ctx, fixture.projectID, "getting-started"))
	if loaded.ID != pageID || !strings.Contains(loaded.Content, "Getting Started") {
		t.Fatalf("unexpected loaded wiki page: %+v", loaded)
	}
}

func assertUpdateWikiPage(t *testing.T, fixture wikiFixture) {
	t.Helper()

	updatedTitle := "Getting Started v2"
	updatedContent := "# Getting Started v2\n\nUpdated."
	updated := testutil.Must(fixture.service.UpdatePage(fixture.ctx, fixture.projectID, "getting-started", wikiservice.UpdatePageInput{
		Title:        &updatedTitle,
		Content:      &updatedContent,
		EditorUserID: fixture.authorID,
	}))
	if updated.Title != updatedTitle || updated.Content != updatedContent || updated.LastEditedByUserID != fixture.authorID {
		t.Fatalf("unexpected updated wiki page: %+v", updated)
	}
}

func assertDeleteWikiPage(t *testing.T, fixture wikiFixture, pageID int64) {
	t.Helper()

	deleted := testutil.Must(fixture.service.DeletePage(fixture.ctx, fixture.projectID, "getting-started"))
	if deleted.ID != pageID {
		t.Fatalf("unexpected deleted wiki page: %+v", deleted)
	}
	if _, getPageErr := fixture.service.GetPage(fixture.ctx, fixture.projectID, "getting-started"); getPageErr == nil {
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
