package mergerequest

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
	projectmergerequestrepo "github.com/DaiYuANg/gity/internal/repository/projectmergerequest"
	userrepo "github.com/DaiYuANg/gity/internal/repository/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/repository/usertoken"
	namespaceservice "github.com/DaiYuANg/gity/internal/service/namespace"
	projectservice "github.com/DaiYuANg/gity/internal/service/project"
	userservice "github.com/DaiYuANg/gity/internal/service/user"
	_ "modernc.org/sqlite"
)

func TestMergeRequestFlow(t *testing.T) {
	t.Parallel()

	dbPath := filepath.Join(t.TempDir(), "gity-mr-test.db")
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
	namespaceRepository, _ := namespacerepo.NewRepository(db)
	namespaceMemberRepository, _ := namespacememberrepo.NewRepository(db)
	projectRepository, _ := projectrepo.NewRepository(db)
	mergeRequestRepository, _ := projectmergerequestrepo.NewRepository(db)
	userRepository, _ := userrepo.NewRepository(db)
	userTokenRepository, _ := usertokenrepo.NewRepository(db)

	repoRoot := filepath.Join(t.TempDir(), "repos")
	runner := gitexec.NewRunner(config.Settings{Git: config.GitSettings{Bin: "git", RepoRoot: repoRoot}})
	gitRepository := gitrepo.NewService(config.Settings{Git: config.GitSettings{RepoRoot: repoRoot}})

	userSvc := userservice.NewService(logger, userRepository, userTokenRepository)
	namespaceSvc := namespaceservice.NewService(logger, namespaceRepository, namespaceMemberRepository, userRepository)
	projectSvc := projectservice.NewService(logger, projectRepository, runner, gitRepository, namespaceRepository)
	mergeRequestSvc := NewService(projectRepository, mergeRequestRepository, userRepository, gitRepository)

	owner, err := userSvc.Create(ctx, userservice.CreateInput{Username: "alice", DisplayName: "Alice", Email: "alice@gity.dev"})
	if err != nil {
		t.Fatalf("create user: %v", err)
	}
	space, err := namespaceSvc.Create(ctx, namespaceservice.CreateInput{Kind: "group", Name: "Core Team", PathKey: "core-team", OwnerUserID: owner.ID})
	if err != nil {
		t.Fatalf("create namespace: %v", err)
	}
	project, err := projectSvc.Create(ctx, projectservice.CreateInput{NamespaceID: space.ID, Name: "Gity", PathKey: "gity", DefaultBranch: "main", Visibility: "private"})
	if err != nil {
		t.Fatalf("create project: %v", err)
	}
	if err := pushFixtureBranches(ctx, repoRoot, project.FullPath+".git"); err != nil {
		t.Fatalf("push fixture branches: %v", err)
	}

	mr, err := mergeRequestSvc.Create(ctx, project.ID, CreateInput{AuthorUserID: owner.ID, Title: "merge feature", Description: "merge feature into main", SourceBranch: "feature", TargetBranch: "main"})
	if err != nil {
		t.Fatalf("create merge request: %v", err)
	}
	if mr.IID != 1 {
		t.Fatalf("expected first merge request iid to be 1, got %d", mr.IID)
	}

	updated, err := mergeRequestSvc.Update(ctx, project.ID, mr.IID, UpdateInput{State: stringPtr("closed")})
	if err != nil {
		t.Fatalf("update merge request: %v", err)
	}
	if updated.State != "closed" {
		t.Fatalf("unexpected merge request state: %s", updated.State)
	}

	items, err := mergeRequestSvc.List(ctx, project.ID)
	if err != nil {
		t.Fatalf("list merge requests: %v", err)
	}
	if len(items) != 1 || items[0].SourceBranch != "feature" || items[0].TargetBranch != "main" {
		t.Fatalf("unexpected merge requests: %+v", items)
	}
}

func stringPtr(value string) *string {
	return &value
}

func pushFixtureBranches(ctx context.Context, repoRoot string, repoPath string) error {
	worktree := filepath.Join(filepath.Dir(repoRoot), "fixture-worktree-mr")
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
	if err := runGit(ctx, worktree, "add", "."); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "commit", "-m", "Initial repository content"); err != nil {
		return err
	}
	if err := runGit(ctx, worktree, "branch", "feature"); err != nil {
		return err
	}

	absRepo := filepath.Join(repoRoot, filepath.FromSlash(repoPath))
	repoURL := "file:///" + filepath.ToSlash(absRepo)
	if err := runGit(ctx, worktree, "push", repoURL, "main:refs/heads/main"); err != nil {
		return err
	}
	return runGit(ctx, worktree, "push", repoURL, "feature:refs/heads/feature")
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
