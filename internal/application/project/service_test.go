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

	fixture := newProjectFixture(t)
	assertCreateNamespace(t, fixture)
	assertCreateProject(t, fixture)
	seedProjectRepository(t, fixture)
	assertNamespaceProjectLists(t, fixture)
	assertNamespaceMembers(t, fixture)
	assertProjectBranchWorkflow(t, fixture)
	assertProjectRepositoryContent(t, fixture)
}

type projectFixture struct {
	ctx              context.Context
	repoRoot         string
	ownerID          int64
	namespaceID      int64
	projectID        int64
	projectFullPath  string
	namespaceService *namespaceservice.Service
	projectService   *projectservice.Service
}

func newProjectFixture(t *testing.T) *projectFixture {
	t.Helper()

	dbPath := filepath.Join(t.TempDir(), "gity-test.db")
	db, err := dbx.Open(
		dbx.WithDriver("sqlite"),
		dbx.WithDSN(fmt.Sprintf("file:%s?_pragma=foreign_keys(1)", dbPath)),
		dbx.WithDialect(sqliteDialect.New()),
	)
	testutil.RequireNoError(t, err, "open db")
	testutil.CleanupClose(t, "db", db)

	ctx := context.Background()
	testutil.RequireNoError(t, core.EnsureSchema(ctx, db), "ensure schema")

	logger := slog.Default()
	namespaceRepository := testutil.Must(namespacerepo.NewRepository(db))
	namespaceMemberRepository := testutil.Must(namespacememberrepo.NewRepository(db))
	projectRepository := testutil.Must(projectrepo.NewRepository(db))
	projectBranchProtectionRepository := testutil.Must(projectbranchprotectionrepo.NewRepository(db))
	userRepository := testutil.Must(userrepo.NewRepository(db))
	userTokenRepository := testutil.Must(usertokenrepo.NewRepository(db))
	repoRoot := filepath.Join(t.TempDir(), "repos")
	settings := config.Settings{Git: config.GitSettings{Bin: "git", RepoRoot: repoRoot}}
	runner := gitexec.NewRunner(settings)
	gitRepository := gitrepo.NewService(settings)

	userSvc := userservice.NewService(logger, userRepository, userTokenRepository)
	namespaceSvc := namespaceservice.NewService(logger, namespaceRepository, namespaceMemberRepository, userRepository)
	projectSvc := projectservice.NewService(logger, projectRepository, runner, gitRepository, namespaceRepository, projectBranchProtectionRepository)

	owner := testutil.Must(userSvc.Create(ctx, userservice.CreateInput{
		Username:    "alice",
		DisplayName: "Alice",
		Email:       "alice@gity.dev",
	}))

	return &projectFixture{
		ctx:              ctx,
		repoRoot:         repoRoot,
		ownerID:          owner.ID,
		namespaceService: namespaceSvc,
		projectService:   projectSvc,
	}
}

func assertCreateNamespace(t *testing.T, fixture *projectFixture) {
	t.Helper()

	namespace := testutil.Must(fixture.namespaceService.Create(fixture.ctx, namespaceservice.CreateInput{
		Kind:        "group",
		Name:        "Core Team",
		PathKey:     "core-team",
		OwnerUserID: fixture.ownerID,
		Description: "Core platform namespace",
	}))
	if namespace.ID == 0 {
		t.Fatalf("expected namespace id to be assigned")
	}
	if namespace.FullPath != "core-team" {
		t.Fatalf("unexpected namespace full path: %s", namespace.FullPath)
	}
	fixture.namespaceID = namespace.ID
}

func assertCreateProject(t *testing.T, fixture *projectFixture) {
	t.Helper()

	project := testutil.Must(fixture.projectService.Create(fixture.ctx, projectservice.CreateInput{
		NamespaceID:   fixture.namespaceID,
		Name:          "Gity",
		PathKey:       "gity",
		Visibility:    "private",
		Description:   "Git hosting platform",
		DefaultBranch: "main",
	}))
	if project.ID == 0 {
		t.Fatalf("expected project id to be assigned")
	}
	if project.FullPath != "core-team/gity" {
		t.Fatalf("unexpected project full path: %s", project.FullPath)
	}
	if project.Visibility != "private" {
		t.Fatalf("unexpected project visibility: %s", project.Visibility)
	}
	if _, statErr := os.Stat(filepath.Join(fixture.repoRoot, "core-team", "gity.git")); statErr != nil {
		t.Fatalf("expected bare repo to exist: %v", statErr)
	}
	fixture.projectID = project.ID
	fixture.projectFullPath = project.FullPath
}

func seedProjectRepository(t *testing.T, fixture *projectFixture) {
	t.Helper()
	testutil.RequireNoError(t, pushFixtureCommit(fixture.ctx, fixture.repoRoot, fixture.projectFullPath+".git"), "push fixture commit")
}

func assertNamespaceProjectLists(t *testing.T, fixture *projectFixture) {
	t.Helper()

	namespaces := testutil.Must(fixture.namespaceService.List(fixture.ctx))
	if namespaces.Len() != 1 {
		t.Fatalf("expected one namespace, got %d", namespaces.Len())
	}

	projects := testutil.Must(fixture.projectService.List(fixture.ctx, &fixture.namespaceID))
	if projects.Len() != 1 {
		t.Fatalf("expected one project, got %d", projects.Len())
	}
}

func assertNamespaceMembers(t *testing.T, fixture *projectFixture) {
	t.Helper()

	members := testutil.Must(fixture.namespaceService.ListMembers(fixture.ctx, fixture.namespaceID))
	if len(members) != 1 || members[0].Role != "owner" || members[0].UserID != fixture.ownerID {
		t.Fatalf("unexpected namespace members: %+v", members)
	}
}

func assertProjectBranchWorkflow(t *testing.T, fixture *projectFixture) {
	t.Helper()

	branches := testutil.Must(fixture.projectService.ListBranches(fixture.ctx, fixture.projectID))
	if len(branches) != 1 || branches[0].Name != "main" {
		t.Fatalf("unexpected branches: %+v", branches)
	}
	featureBranch := testutil.Must(fixture.projectService.CreateBranch(fixture.ctx, fixture.projectID, "feature/editor", "main"))
	if featureBranch.Name != "feature/editor" || featureBranch.LastCommitSHA == "" {
		t.Fatalf("unexpected created branch: %+v", featureBranch)
	}
	testutil.RequireNoError(t, fixture.projectService.CreateFileCommit(fixture.ctx, fixture.projectID, projectservice.CreateFileCommitInput{
		BranchName: "feature/editor",
		Path:       "docs/new.md",
		Content:    "New file from API\n",
		Message:    "Add new file",
	}), "create file commit")
	createdBlob := testutil.Must(fixture.projectService.GetBlob(fixture.ctx, fixture.projectID, "feature/editor", "docs/new.md"))
	if createdBlob.Content != "New file from API\n" {
		t.Fatalf("unexpected created blob: %+v", createdBlob)
	}
	assertProjectBranchProtection(t, fixture)
}

func assertProjectBranchProtection(t *testing.T, fixture *projectFixture) {
	t.Helper()

	protectedBranch := testutil.Must(fixture.projectService.SetBranchProtection(fixture.ctx, fixture.projectID, "feature/editor", true))
	if !protectedBranch.IsProtected {
		t.Fatalf("expected branch to be protected: %+v", protectedBranch)
	}
	if createCommitErr := fixture.projectService.CreateFileCommit(fixture.ctx, fixture.projectID, projectservice.CreateFileCommitInput{
		BranchName: "feature/editor",
		Path:       "docs/protected.md",
		Content:    "blocked\n",
		Message:    "Try protected branch",
	}); createCommitErr == nil {
		t.Fatalf("expected protected branch file commit to fail")
	}
}

func assertProjectRepositoryContent(t *testing.T, fixture *projectFixture) {
	t.Helper()

	commits := testutil.Must(fixture.projectService.ListCommits(fixture.ctx, fixture.projectID, "", 10))
	if len(commits) != 1 || commits[0].Message != "Initial repository content" {
		t.Fatalf("unexpected commits: %+v", commits)
	}
	assertProjectReadmeAndLanguages(t, fixture)
	assertProjectTreeAndBlob(t, fixture)
}

func assertProjectReadmeAndLanguages(t *testing.T, fixture *projectFixture) {
	t.Helper()

	readme := testutil.Must(fixture.projectService.GetReadme(fixture.ctx, fixture.projectID, ""))
	if readme.Path != "README.md" || !strings.Contains(readme.Content, "Hello Gity") {
		t.Fatalf("unexpected readme blob: %+v", readme)
	}
	languages := testutil.Must(fixture.projectService.AnalyzeLanguages(fixture.ctx, fixture.projectID, "main"))
	if languages.TotalBytes == 0 || len(languages.Languages) == 0 || languages.Languages[0].Language != "Markdown" {
		t.Fatalf("unexpected language analysis: %+v", languages)
	}
}

func assertProjectTreeAndBlob(t *testing.T, fixture *projectFixture) {
	t.Helper()

	rootTree := testutil.Must(fixture.projectService.ListTree(fixture.ctx, fixture.projectID, "", ""))
	if len(rootTree) != 2 {
		t.Fatalf("expected two root entries, got %d", len(rootTree))
	}

	docsTree := testutil.Must(fixture.projectService.ListTree(fixture.ctx, fixture.projectID, "", "docs"))
	if len(docsTree) != 1 || docsTree[0].Path != "docs/guide.md" {
		t.Fatalf("unexpected docs tree: %+v", docsTree)
	}

	blob := testutil.Must(fixture.projectService.GetBlob(fixture.ctx, fixture.projectID, "", "docs/guide.md"))
	if !strings.Contains(blob.Content, "Repository guide") {
		t.Fatalf("unexpected blob content: %+v", blob)
	}
}
