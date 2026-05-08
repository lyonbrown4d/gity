package project_test

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"path/filepath"
	"strings"
	"testing"

	namespaceservice "github.com/DaiYuANg/gity/internal/application/namespace"
	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	userservice "github.com/DaiYuANg/gity/internal/application/user"
	"github.com/DaiYuANg/gity/internal/config"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_exec"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_repo"
	"github.com/DaiYuANg/gity/internal/infrastructure/persistence/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespace"
	namespacememberrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespace_member"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectbranchprotectionrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_branch_protection"
	userrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user_token"
	"github.com/DaiYuANg/gity/internal/testutil"

	"github.com/arcgolabs/dbx"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
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
	testutil.CleanupClose(t, "db", db)

	ctx := context.Background()
	if schemaErr := core.EnsureSchema(ctx, db); schemaErr != nil {
		t.Fatalf("ensure schema: %v", schemaErr)
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
	projectBranchProtectionRepository, err := projectbranchprotectionrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new project branch protection repo: %v", err)
	}
	userRepository, err := userrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new user repo: %v", err)
	}
	userTokenRepository, err := usertokenrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new user token repo: %v", err)
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

	userSvc := userservice.NewService(logger, userRepository, userTokenRepository)
	namespaceSvc := namespaceservice.NewService(logger, namespaceRepository, namespaceMemberRepository, userRepository)
	projectSvc := projectservice.NewService(logger, projectRepository, runner, gitRepository, namespaceRepository, projectBranchProtectionRepository)

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
	if _, statErr := os.Stat(filepath.Join(repoRoot, "core-team", "gity.git")); statErr != nil {
		t.Fatalf("expected bare repo to exist: %v", statErr)
	}
	if pushErr := pushFixtureCommit(ctx, repoRoot, project.FullPath+".git"); pushErr != nil {
		t.Fatalf("push fixture commit: %v", pushErr)
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
	featureBranch, err := projectSvc.CreateBranch(ctx, project.ID, "feature/editor", "main")
	if err != nil {
		t.Fatalf("create branch: %v", err)
	}
	if featureBranch.Name != "feature/editor" || featureBranch.LastCommitSHA == "" {
		t.Fatalf("unexpected created branch: %+v", featureBranch)
	}
	if createCommitErr := projectSvc.CreateFileCommit(ctx, project.ID, projectservice.CreateFileCommitInput{
		BranchName: "feature/editor",
		Path:       "docs/new.md",
		Content:    "New file from API\n",
		Message:    "Add new file",
	}); createCommitErr != nil {
		t.Fatalf("create file commit: %v", createCommitErr)
	}
	createdBlob, err := projectSvc.GetBlob(ctx, project.ID, "feature/editor", "docs/new.md")
	if err != nil {
		t.Fatalf("get created blob: %v", err)
	}
	if createdBlob.Content != "New file from API\n" {
		t.Fatalf("unexpected created blob: %+v", createdBlob)
	}
	protectedBranch, err := projectSvc.SetBranchProtection(ctx, project.ID, "feature/editor", true)
	if err != nil {
		t.Fatalf("protect branch: %v", err)
	}
	if !protectedBranch.IsProtected {
		t.Fatalf("expected branch to be protected: %+v", protectedBranch)
	}
	if createCommitErr := projectSvc.CreateFileCommit(ctx, project.ID, projectservice.CreateFileCommitInput{
		BranchName: "feature/editor",
		Path:       "docs/protected.md",
		Content:    "blocked\n",
		Message:    "Try protected branch",
	}); createCommitErr == nil {
		t.Fatalf("expected protected branch file commit to fail")
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
	languages, err := projectSvc.AnalyzeLanguages(ctx, project.ID, "main")
	if err != nil {
		t.Fatalf("analyze languages: %v", err)
	}
	if languages.TotalBytes == 0 || len(languages.Languages) == 0 || languages.Languages[0].Language != "Markdown" {
		t.Fatalf("unexpected language analysis: %+v", languages)
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
