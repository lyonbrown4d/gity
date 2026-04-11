package project_test

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"

	"github.com/DaiYuANg/arcgo/dbx"
	sqliteDialect "github.com/DaiYuANg/arcgo/dbx/dialect/sqlite"
	"github.com/DaiYuANg/gity/internal/config"
	"github.com/DaiYuANg/gity/internal/platform/gitexec"
	"github.com/DaiYuANg/gity/internal/platform/gitrepo"
	"github.com/DaiYuANg/gity/internal/repository/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/repository/namespace"
	namespacememberrepo "github.com/DaiYuANg/gity/internal/repository/namespacemember"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	userrepo "github.com/DaiYuANg/gity/internal/repository/user"
	namespaceservice "github.com/DaiYuANg/gity/internal/service/namespace"
	projectservice "github.com/DaiYuANg/gity/internal/service/project"
	userservice "github.com/DaiYuANg/gity/internal/service/user"
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
	namespaceMemberRepository, err := namespacememberrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new namespace member repo: %v", err)
	}
	projectRepository, err := projectrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new project repo: %v", err)
	}
	userRepository, err := userrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new user repo: %v", err)
	}
	repoRoot := filepath.Join(t.TempDir(), "repos")
	runner := gitexec.NewRunner(config.Settings{
		Git: config.GitSettings{
			Bin:      "git",
			RepoRoot: repoRoot,
		},
	})
	gitRepository := gitrepo.NewService(config.Settings{
		Git: config.GitSettings{
			RepoRoot: repoRoot,
		},
	})

	userSvc := userservice.NewService(logger, userRepository)
	namespaceSvc := namespaceservice.NewService(logger, namespaceRepository, namespaceMemberRepository, userRepository)
	projectSvc := projectservice.NewService(logger, projectRepository, runner, gitRepository, namespaceRepository)

	owner, err := userSvc.Create(ctx, userservice.CreateInput{
		Username:    "alice",
		DisplayName: "Alice",
		Email:       "alice@gity.dev",
	})
	if err != nil {
		t.Fatalf("create owner user: %v", err)
	}

	namespace, err := namespaceSvc.Create(ctx, namespaceservice.CreateInput{
		Kind:        "group",
		Name:        "Core Team",
		PathKey:     "core-team",
		OwnerUserID: owner.ID,
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
		Visibility:    "private",
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
	if project.Visibility != "private" {
		t.Fatalf("unexpected project visibility: %s", project.Visibility)
	}
	if _, err := os.Stat(filepath.Join(repoRoot, "core-team", "gity.git")); err != nil {
		t.Fatalf("expected bare repo to exist: %v", err)
	}
	if err := pushFixtureCommit(ctx, repoRoot, project.FullPath+".git"); err != nil {
		t.Fatalf("push fixture commit: %v", err)
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

	members, err := namespaceSvc.ListMembers(ctx, namespace.ID)
	if err != nil {
		t.Fatalf("list namespace members: %v", err)
	}
	if len(members) != 1 || members[0].Role != "owner" || members[0].UserID != owner.ID {
		t.Fatalf("unexpected namespace members: %+v", members)
	}

	branches, err := projectSvc.ListBranches(ctx, project.ID)
	if err != nil {
		t.Fatalf("list branches: %v", err)
	}
	if len(branches) != 1 || branches[0].Name != "main" {
		t.Fatalf("unexpected branches: %+v", branches)
	}

	commits, err := projectSvc.ListCommits(ctx, project.ID, "", 10)
	if err != nil {
		t.Fatalf("list commits: %v", err)
	}
	if len(commits) != 1 || commits[0].Message != "Initial repository content" {
		t.Fatalf("unexpected commits: %+v", commits)
	}

	readme, err := projectSvc.GetReadme(ctx, project.ID, "")
	if err != nil {
		t.Fatalf("get readme: %v", err)
	}
	if readme.Path != "README.md" || !strings.Contains(readme.Content, "Hello Gity") {
		t.Fatalf("unexpected readme blob: %+v", readme)
	}

	rootTree, err := projectSvc.ListTree(ctx, project.ID, "", "")
	if err != nil {
		t.Fatalf("list root tree: %v", err)
	}
	if len(rootTree) != 2 {
		t.Fatalf("expected two root entries, got %d", len(rootTree))
	}

	docsTree, err := projectSvc.ListTree(ctx, project.ID, "", "docs")
	if err != nil {
		t.Fatalf("list docs tree: %v", err)
	}
	if len(docsTree) != 1 || docsTree[0].Path != "docs/guide.md" {
		t.Fatalf("unexpected docs tree: %+v", docsTree)
	}

	blob, err := projectSvc.GetBlob(ctx, project.ID, "", "docs/guide.md")
	if err != nil {
		t.Fatalf("get blob: %v", err)
	}
	if !strings.Contains(blob.Content, "Repository guide") {
		t.Fatalf("unexpected blob content: %+v", blob)
	}
}

func pushFixtureCommit(ctx context.Context, repoRoot string, repoPath string) error {
	worktree := filepath.Join(filepath.Dir(repoRoot), "fixture-worktree")
	if err := os.MkdirAll(worktree, 0o755); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "init", "-b", "main"); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "config", "user.name", "Gity Test"); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "config", "user.email", "test@gity.dev"); err != nil {
		return err
	}
	if err := os.WriteFile(filepath.Join(worktree, "README.md"), []byte("# Hello Gity\n"), 0o644); err != nil {
		return err
	}
	if err := os.MkdirAll(filepath.Join(worktree, "docs"), 0o755); err != nil {
		return err
	}
	if err := os.WriteFile(filepath.Join(worktree, "docs", "guide.md"), []byte("Repository guide\n"), 0o644); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "add", "."); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "commit", "-m", "Initial repository content"); err != nil {
		return err
	}

	absRepo := filepath.Join(repoRoot, filepath.FromSlash(repoPath))
	repoURL := "file:///" + filepath.ToSlash(absRepo)
	return runGit(ctx, worktree, "push", repoURL, "HEAD:refs/heads/main")
}

func runGit(ctx context.Context, dir string, args ...string) error {
	cmd := exec.CommandContext(ctx, "git", args...)
	cmd.Dir = dir
	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("git %v: %w: %s", args, err, strings.TrimSpace(string(output)))
	}
	return nil
}
