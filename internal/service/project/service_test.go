package project_test

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"path/filepath"
	"testing"

	"github.com/DaiYuANg/arcgo/dbx"
	sqliteDialect "github.com/DaiYuANg/arcgo/dbx/dialect/sqlite"
	"github.com/DaiYuANg/gity/internal/config"
	"github.com/DaiYuANg/gity/internal/platform/gitexec"
	"github.com/DaiYuANg/gity/internal/repository/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/repository/namespace"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	namespaceservice "github.com/DaiYuANg/gity/internal/service/namespace"
	projectservice "github.com/DaiYuANg/gity/internal/service/project"
	_ "modernc.org/sqlite"
)

func TestNamespaceProjectFlow(t *testing.T) {
	t.Parallel()

	dbPath := filepath.Join(t.TempDir(), "gity-test.db")
	db, err := dbx.Open(
		dbx.WithDriver("sqlite"),
		dbx.WithDSN(fmt.Sprintf("file:%s?_pragma=foreign_keys(1)", dbPath)),
		dbx.WithDialect(sqliteDialect.New()),
	)
	if err != nil {
		t.Fatalf("open db: %v", err)
	}
	defer db.Close()

	ctx := context.Background()
	if err := core.EnsureSchema(ctx, db); err != nil {
		t.Fatalf("ensure schema: %v", err)
	}

	logger := slog.Default()
	namespaceRepository, err := namespacerepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new namespace repo: %v", err)
	}
	projectRepository, err := projectrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new project repo: %v", err)
	}
	repoRoot := filepath.Join(t.TempDir(), "repos")
	runner := gitexec.NewRunner(config.Settings{
		Git: config.GitSettings{
			Bin:      "git",
			RepoRoot: repoRoot,
		},
	})

	namespaceSvc := namespaceservice.NewService(logger, namespaceRepository)
	projectSvc := projectservice.NewService(logger, projectRepository, runner, namespaceRepository)

	namespace, err := namespaceSvc.Create(ctx, namespaceservice.CreateInput{
		Kind:        "group",
		Name:        "Core Team",
		PathKey:     "core-team",
		Description: "Core platform namespace",
	})
	if err != nil {
		t.Fatalf("create namespace: %v", err)
	}
	if namespace.ID == 0 {
		t.Fatalf("expected namespace id to be assigned")
	}
	if namespace.FullPath != "core-team" {
		t.Fatalf("unexpected namespace full path: %s", namespace.FullPath)
	}

	project, err := projectSvc.Create(ctx, projectservice.CreateInput{
		NamespaceID:   namespace.ID,
		Name:          "Gity",
		PathKey:       "gity",
		Description:   "Git hosting platform",
		DefaultBranch: "main",
	})
	if err != nil {
		t.Fatalf("create project: %v", err)
	}
	if project.ID == 0 {
		t.Fatalf("expected project id to be assigned")
	}
	if project.FullPath != "core-team/gity" {
		t.Fatalf("unexpected project full path: %s", project.FullPath)
	}
	if _, err := os.Stat(filepath.Join(repoRoot, "core-team", "gity.git")); err != nil {
		t.Fatalf("expected bare repo to exist: %v", err)
	}

	namespaces, err := namespaceSvc.List(ctx)
	if err != nil {
		t.Fatalf("list namespaces: %v", err)
	}
	if namespaces.Len() != 1 {
		t.Fatalf("expected one namespace, got %d", namespaces.Len())
	}

	projects, err := projectSvc.List(ctx, &namespace.ID)
	if err != nil {
		t.Fatalf("list projects: %v", err)
	}
	if projects.Len() != 1 {
		t.Fatalf("expected one project, got %d", projects.Len())
	}
}
