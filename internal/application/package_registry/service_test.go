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
	"sync"
	"testing"
	"time"

	organizationservice "github.com/lyonbrown4d/gity/internal/application/organization"
	packageregistryservice "github.com/lyonbrown4d/gity/internal/application/package_registry"
	storageports "github.com/lyonbrown4d/gity/internal/application/ports"
	projectservice "github.com/lyonbrown4d/gity/internal/application/project"
	userservice "github.com/lyonbrown4d/gity/internal/application/user"
	"github.com/lyonbrown4d/gity/internal/config"
	packagedomain "github.com/lyonbrown4d/gity/internal/domain/package_registry"
	"github.com/lyonbrown4d/gity/internal/infrastructure/git_exec"
	"github.com/lyonbrown4d/gity/internal/infrastructure/git_repo"
	"github.com/lyonbrown4d/gity/internal/infrastructure/persistence/core"
	organizationrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/organization"
	organizationmemberrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/organization_member"
	projectrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project"
	projectbranchprotectionrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_branch_protection"
	projectpackagerepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_package"
	projectpackagefilerepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_package_file"
	projectpackageversionrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_package_version"
	userrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/user_token"
	infrastorage "github.com/lyonbrown4d/gity/internal/infrastructure/storage"
	"github.com/lyonbrown4d/gity/internal/testutil"

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

func TestPackageFileMetadataDoesNotLoadBlob(t *testing.T) {
	t.Parallel()

	fixture := newPackageRegistryFixture(t)
	fileRecord := uploadPackageFile(t, fixture)
	fixture.storage.resetLoadCount()

	byID := testutil.Must(fixture.service.GetFileMetadata(fixture.ctx, fixture.projectID, fileRecord.ID))
	byCoordinate := testutil.Must(fixture.service.GetFileMetadataByCoordinate(
		fixture.ctx,
		fixture.projectID,
		"maven",
		"io.gity:gity-api",
		"1.0.0",
		"io/gity/gity-api/1.0.0/gity-api-1.0.0.jar",
	))

	if byID.File.ID != fileRecord.ID || byCoordinate.File.ID != fileRecord.ID {
		t.Fatalf("unexpected package file metadata: by_id=%+v by_coordinate=%+v", byID, byCoordinate)
	}
	if byID.ETag == "" || byID.ETag != byCoordinate.ETag {
		t.Fatalf("expected stable non-empty ETag, by_id=%q by_coordinate=%q", byID.ETag, byCoordinate.ETag)
	}
	if want := fileRecord.UpdatedAt.UTC().Truncate(time.Second); !byID.LastModified.Equal(want) {
		t.Fatalf("last modified = %s, want %s", byID.LastModified, want)
	}
	if got := fixture.storage.loadCount(); got != 0 {
		t.Fatalf("metadata lookup loaded blob %d times", got)
	}
}

func TestPackageFileMetadataETagChangesWhenFileChanges(t *testing.T) {
	t.Parallel()

	fixture := newPackageRegistryFixture(t)
	firstFile := uploadPackageFile(t, fixture)
	first := testutil.Must(fixture.service.GetFileMetadata(fixture.ctx, fixture.projectID, firstFile.ID))
	repeated := testutil.Must(fixture.service.GetFileMetadata(fixture.ctx, fixture.projectID, firstFile.ID))
	if first.ETag != repeated.ETag {
		t.Fatalf("ETag changed for unchanged file: first=%q repeated=%q", first.ETag, repeated.ETag)
	}

	testutil.RequireNoError(t, fixture.fileRepo.MarkStored(fixture.ctx, firstFile.ID, storageports.StoreProjectPackageFileInput{
		ContentType: "application/java-archive",
		ByteSize:    firstFile.ByteSize + 1,
		StorageKey:  firstFile.StorageKey + ".changed",
	}), "change package file metadata")
	changed := testutil.Must(fixture.service.GetFileMetadata(fixture.ctx, fixture.projectID, firstFile.ID))
	if first.ETag == changed.ETag {
		t.Fatalf("ETag did not change after file replacement: %q", first.ETag)
	}
}

type packageRegistryFixture struct {
	ctx             context.Context
	repoRoot        string
	projectID       int64
	projectFullPath string
	service         *packageregistryservice.Service
	storage         *countingObjectStorage
	fileRepo        *projectpackagefilerepo.Repository
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
	organizationRepository := testutil.Must(organizationrepo.NewRepository(db))
	organizationMemberRepository := testutil.Must(organizationmemberrepo.NewRepository(db))
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
	storageBackend := testutil.Must(infrastorage.NewService(config.Settings{Storage: config.StorageSettings{Driver: "local", Root: storageRoot}}))
	storage := &countingObjectStorage{ObjectStorage: storageBackend}

	userSvc := userservice.NewService(logger, userRepository, userTokenRepository)
	organizationSvc := organizationservice.NewService(logger, organizationRepository, organizationMemberRepository, userRepository)
	projectSvc := projectservice.NewService(logger, projectRepository, runner, gitRepository, organizationRepository, projectBranchProtectionRepository)
	packageSvc := packageregistryservice.NewService(projectRepository, packageRepository, versionRepository, fileRepository, storage)

	owner := testutil.Must(userSvc.Create(ctx, userservice.CreateInput{Username: "alice", DisplayName: "Alice", Email: "alice@gity.dev"}))
	space := testutil.Must(organizationSvc.Create(ctx, organizationservice.CreateInput{Name: "Core Team", PathKey: "core-team", OwnerUserID: owner.ID}))
	project := testutil.Must(projectSvc.Create(ctx, projectservice.CreateInput{OrganizationID: space.ID, Name: "Gity", PathKey: "gity", DefaultBranch: "main", Visibility: "private"}))
	return packageRegistryFixture{
		ctx:             ctx,
		repoRoot:        repoRoot,
		projectID:       project.ID,
		projectFullPath: project.FullPath,
		service:         packageSvc,
		storage:         storage,
		fileRepo:        fileRepository,
	}
}

type countingObjectStorage struct {
	storageports.ObjectStorage
	mu        sync.Mutex
	loadCalls int
}

func (s *countingObjectStorage) Load(ctx context.Context, key string) ([]byte, error) {
	s.mu.Lock()
	s.loadCalls++
	s.mu.Unlock()
	return s.ObjectStorage.Load(ctx, key)
}

func (s *countingObjectStorage) resetLoadCount() {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.loadCalls = 0
}

func (s *countingObjectStorage) loadCount() int {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.loadCalls
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
