package project_test

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"path/filepath"
	"strings"
	"testing"

	organizationservice "github.com/DaiYuANg/gity/internal/application/organization"
	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	userservice "github.com/DaiYuANg/gity/internal/application/user"
	"github.com/DaiYuANg/gity/internal/config"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_exec"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_repo"
	"github.com/DaiYuANg/gity/internal/infrastructure/persistence/core"
	organizationrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/organization"
	organizationmemberrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/organization_member"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectbranchprotectionrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_branch_protection"
	userrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user_token"
	"github.com/DaiYuANg/gity/internal/testutil"

	"github.com/arcgolabs/dbx"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestOrganizationProjectFlow(t *testing.T) {
	t.Parallel()

	fixture := newProjectFixture(t)
	assertCreateOrganization(t, fixture)
	assertCreateProject(t, fixture)
	seedProjectRepository(t, fixture)
	assertOrganizationProjectLists(t, fixture)
	assertOrganizationMembers(t, fixture)
	assertProjectBranchWorkflow(t, fixture)
	assertProjectRepositoryContent(t, fixture)
	assertProjectDeletionGovernance(t, fixture)
}

type projectFixture struct {
	ctx                 context.Context
	repoRoot            string
	ownerID             int64
	organizationID      int64
	projectID           int64
	projectFullPath     string
	organizationService *organizationservice.Service
	projectService      *projectservice.Service
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
	organizationRepository := testutil.Must(organizationrepo.NewRepository(db))
	organizationMemberRepository := testutil.Must(organizationmemberrepo.NewRepository(db))
	projectRepository := testutil.Must(projectrepo.NewRepository(db))
	projectBranchProtectionRepository := testutil.Must(projectbranchprotectionrepo.NewRepository(db))
	userRepository := testutil.Must(userrepo.NewRepository(db))
	userTokenRepository := testutil.Must(usertokenrepo.NewRepository(db))
	repoRoot := filepath.Join(t.TempDir(), "repos")
	settings := config.Settings{Git: config.GitSettings{Bin: "git", RepoRoot: repoRoot}}
	runner := gitexec.NewRunner(settings)
	gitRepository := gitrepo.NewService(settings)

	userSvc := userservice.NewService(logger, userRepository, userTokenRepository)
	organizationSvc := organizationservice.NewService(logger, organizationRepository, organizationMemberRepository, userRepository)
	projectSvc := projectservice.NewService(logger, projectRepository, runner, gitRepository, organizationRepository, projectBranchProtectionRepository)

	owner := testutil.Must(userSvc.Create(ctx, userservice.CreateInput{
		Username:    "alice",
		DisplayName: "Alice",
		Email:       "alice@gity.dev",
	}))

	return &projectFixture{
		ctx:                 ctx,
		repoRoot:            repoRoot,
		ownerID:             owner.ID,
		organizationService: organizationSvc,
		projectService:      projectSvc,
	}
}

func assertCreateOrganization(t *testing.T, fixture *projectFixture) {
	t.Helper()

	organization := testutil.Must(fixture.organizationService.Create(fixture.ctx, organizationservice.CreateInput{

		Name:        "Core Team",
		PathKey:     "core-team",
		OwnerUserID: fixture.ownerID,
		Description: "Core platform organization",
	}))
	if organization.ID == 0 {
		t.Fatalf("expected organization id to be assigned")
	}
	if organization.FullPath != "core-team" {
		t.Fatalf("unexpected organization full path: %s", organization.FullPath)
	}
	fixture.organizationID = organization.ID
}

func assertCreateProject(t *testing.T, fixture *projectFixture) {
	t.Helper()

	project := testutil.Must(fixture.projectService.Create(fixture.ctx, projectservice.CreateInput{
		OrganizationID: fixture.organizationID,
		Name:           "Gity",
		PathKey:        "gity",
		Visibility:     "private",
		Description:    "Git hosting platform",
		DefaultBranch:  "main",
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

func assertOrganizationProjectLists(t *testing.T, fixture *projectFixture) {
	t.Helper()

	organizations := testutil.Must(fixture.organizationService.List(fixture.ctx))
	if organizations.Len() != 1 {
		t.Fatalf("expected one organization, got %d", organizations.Len())
	}

	projects := testutil.Must(fixture.projectService.List(fixture.ctx, &fixture.organizationID))
	if projects.Len() != 1 {
		t.Fatalf("expected one project, got %d", projects.Len())
	}
}

func assertOrganizationMembers(t *testing.T, fixture *projectFixture) {
	t.Helper()

	members := testutil.Must(fixture.organizationService.ListMembers(fixture.ctx, fixture.organizationID))
	if len(members) != 1 || members[0].Role != "owner" || members[0].UserID != fixture.ownerID {
		t.Fatalf("unexpected organization members: %+v", members)
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

	releaseBranch := testutil.Must(fixture.projectService.CreateBranch(fixture.ctx, fixture.projectID, "release/1.0", "main"))
	if releaseBranch.Name != "release/1.0" {
		t.Fatalf("unexpected release branch: %+v", releaseBranch)
	}
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
	pattern := testutil.Must(fixture.projectService.UpsertBranchProtection(fixture.ctx, fixture.projectID, projectservice.BranchProtectionInput{
		BranchName:          "release/*",
		RuleType:            "pattern",
		PushAccessLevel:     "no_one",
		MergeAccessLevel:    "maintainer",
		RequireMergeRequest: true,
	}))
	if pattern.RuleType != "pattern" || pattern.BranchName != "release/*" {
		t.Fatalf("unexpected pattern protection: %+v", pattern)
	}
	branches := testutil.Must(fixture.projectService.ListBranches(fixture.ctx, fixture.projectID))
	if !branchProtected(branches, "release/1.0") {
		t.Fatalf("expected release branch to match pattern protection: %+v", branches)
	}
	if _, err := fixture.projectService.CreateBranch(fixture.ctx, fixture.projectID, "release/2.0", "main"); err == nil {
		t.Fatalf("expected protected pattern branch creation to fail")
	}
	if deleteErr := fixture.projectService.DeleteBranch(fixture.ctx, fixture.projectID, "release/1.0"); deleteErr == nil {
		t.Fatalf("expected protected branch delete to fail")
	}
	testutil.Must(fixture.projectService.UpsertBranchProtection(fixture.ctx, fixture.projectID, projectservice.BranchProtectionInput{
		BranchName:          "release/*",
		RuleType:            "pattern",
		PushAccessLevel:     "no_one",
		MergeAccessLevel:    "maintainer",
		RequireMergeRequest: true,
		AllowDelete:         true,
	}))
	testutil.RequireNoError(t, fixture.projectService.DeleteBranch(fixture.ctx, fixture.projectID, "release/1.0"), "delete protected branch with allow_delete")
	if deleteErr := fixture.projectService.DeleteBranch(fixture.ctx, fixture.projectID, "main"); deleteErr == nil {
		t.Fatalf("expected default branch delete to fail")
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
