package packageregistry_test

import (
	"context"
	"encoding/base64"
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"

	namespaceservice "github.com/DaiYuANg/gity/internal/application/namespace"
	packageregistryservice "github.com/DaiYuANg/gity/internal/application/package_registry"
	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	userservice "github.com/DaiYuANg/gity/internal/application/user"
	"github.com/DaiYuANg/gity/internal/config"
	packagedomain "github.com/DaiYuANg/gity/internal/domain/package_registry"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_exec"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_repo"
	"github.com/DaiYuANg/gity/internal/infrastructure/persistence/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespace"
	namespacememberrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespace_member"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectbranchprotectionrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_branch_protection"
	projectpackagerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_package"
	projectpackagefilerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_package_file"
	projectpackageversionrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_package_version"
	userrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user_token"
	infrastorage "github.com/DaiYuANg/gity/internal/infrastructure/storage"
	"github.com/DaiYuANg/gity/internal/testutil"

	"github.com/arcgolabs/dbx"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestPackageRegistryFlow(t *testing.T) {
	t.Parallel()

	fixture := newPackageRegistryFixture(t)
	testutil.RequireNoError(t, pushFixtureCommit(fixture.ctx, fixture.repoRoot, fixture.projectFullPath+".git"), "push fixture commit")

	fileRecord := uploadPackageFile(t, fixture)
	assertPackageFile(t, fileRecord)
	assertPackageList(t, fixture, fileRecord.ID)
}

type packageRegistryFixture struct {
	ctx             context.Context
	repoRoot        string
	projectID       int64
	projectFullPath string
	service         *packageregistryservice.Service
}

func newPackageRegistryFixture(t *testing.T) packageRegistryFixture {
	t.Helper()
	dbPath := filepath.Join(t.TempDir(), "gity-package-test.db")
	db := testutil.Must(dbx.Open(
		dbx.WithDriver("sqlite"),
		dbx.WithDSN(fmt.Sprintf("file:%s?_pragma=foreign_keys(1)", dbPath)),
		dbx.WithDialect(sqliteDialect.New()),
	))
	testutil.CleanupClose(t, "db", db)

	ctx := context.Background()
	testutil.RequireNoError(t, core.EnsureSchema(ctx, db), "ensure schema")

	logger := slog.Default()
	namespaceRepository := testutil.Must(namespacerepo.NewRepository(db))
	namespaceMemberRepository := testutil.Must(namespacememberrepo.NewRepository(db))
	projectRepository := testutil.Must(projectrepo.NewRepository(db))
	projectBranchProtectionRepository := testutil.Must(projectbranchprotectionrepo.NewRepository(db))
	packageRepository := testutil.Must(projectpackagerepo.NewRepository(db))
	versionRepository := testutil.Must(projectpackageversionrepo.NewRepository(db))
	fileRepository := testutil.Must(projectpackagefilerepo.NewRepository(db))
	userRepository := testutil.Must(userrepo.NewRepository(db))
	userTokenRepository := testutil.Must(usertokenrepo.NewRepository(db))

	repoRoot := filepath.Join(t.TempDir(), "repos")
	storageRoot := filepath.Join(t.TempDir(), "storage")
	runner := gitexec.NewRunner(config.Settings{Git: config.GitSettings{Bin: "git", RepoRoot: repoRoot}})
	gitRepository := gitrepo.NewService(config.Settings{Git: config.GitSettings{RepoRoot: repoRoot}})
	storage := testutil.Must(infrastorage.NewService(config.Settings{Storage: config.StorageSettings{Driver: "local", Root: storageRoot}}))

	userSvc := userservice.NewService(logger, userRepository, userTokenRepository)
	namespaceSvc := namespaceservice.NewService(logger, namespaceRepository, namespaceMemberRepository, userRepository)
	projectSvc := projectservice.NewService(logger, projectRepository, runner, gitRepository, namespaceRepository, projectBranchProtectionRepository)
	packageSvc := packageregistryservice.NewService(projectRepository, packageRepository, versionRepository, fileRepository, storage)

	owner := testutil.Must(userSvc.Create(ctx, userservice.CreateInput{Username: "alice", DisplayName: "Alice", Email: "alice@gity.dev"}))
	space := testutil.Must(namespaceSvc.Create(ctx, namespaceservice.CreateInput{Kind: "group", Name: "Core Team", PathKey: "core-team", OwnerUserID: owner.ID}))
	project := testutil.Must(projectSvc.Create(ctx, projectservice.CreateInput{NamespaceID: space.ID, Name: "Gity", PathKey: "gity", DefaultBranch: "main", Visibility: "private"}))
	return packageRegistryFixture{
		ctx:             ctx,
		repoRoot:        repoRoot,
		projectID:       project.ID,
		projectFullPath: project.FullPath,
		service:         packageSvc,
	}
}

func uploadPackageFile(t *testing.T, fixture packageRegistryFixture) packagedomain.ProjectPackageFile {
	t.Helper()
	return testutil.Must(fixture.service.UploadFile(fixture.ctx, fixture.projectID, packageregistryservice.UploadFileInput{
		Type:          "maven",
		Name:          "io.gity:gity-api",
		Version:       "1.0.0",
		FileName:      "gity-api-1.0.0.jar",
		FilePath:      "io/gity/gity-api/1.0.0/gity-api-1.0.0.jar",
		ContentType:   "application/java-archive",
		ContentBase64: base64.StdEncoding.EncodeToString([]byte("jar-binary")),
	}))
}

func assertPackageFile(t *testing.T, fileRecord packagedomain.ProjectPackageFile) {
	t.Helper()
	if fileRecord.ByteSize != int64(len("jar-binary")) {
		t.Fatalf("unexpected package file size: %d", fileRecord.ByteSize)
	}
}

func assertPackageList(t *testing.T, fixture packageRegistryFixture, fileID int64) {
	t.Helper()
	packages := testutil.Must(fixture.service.ListPackages(fixture.ctx, fixture.projectID))
	if len(packages) != 1 || packages[0].Type != "maven" {
		t.Fatalf("unexpected packages: %+v", packages)
	}
	assertPackageDetail(t, fixture, packages[0].ID)
	assertPackageContent(t, fixture, fileID)
}

func assertPackageDetail(t *testing.T, fixture packageRegistryFixture, packageID int64) {
	t.Helper()
	detail := testutil.Must(fixture.service.GetPackage(fixture.ctx, fixture.projectID, packageID))
	if len(detail.Versions) != 1 || len(detail.Versions[0].Files) != 1 {
		t.Fatalf("unexpected package detail: %+v", detail)
	}
}

func assertPackageContent(t *testing.T, fixture packageRegistryFixture, fileID int64) {
	t.Helper()
	content := testutil.Must(fixture.service.GetFileContent(fixture.ctx, fixture.projectID, fileID))
	decoded := testutil.Must(base64.StdEncoding.DecodeString(content.Content))
	if string(decoded) != "jar-binary" {
		t.Fatalf("unexpected package content: %s", string(decoded))
	}
}

func pushFixtureCommit(ctx context.Context, repoRoot, repoPath string) error {
	worktree := filepath.Join(filepath.Dir(repoRoot), "fixture-worktree-package")
	if err := os.MkdirAll(worktree, 0o750); err != nil {
		return fmt.Errorf("create fixture worktree: %w", err)
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
	if err := os.WriteFile(filepath.Join(worktree, "README.md"), []byte("# Hello Gity\n"), 0o600); err != nil {
		return fmt.Errorf("write fixture readme: %w", err)
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
