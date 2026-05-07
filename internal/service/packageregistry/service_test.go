package packageregistry

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

	"github.com/DaiYuANg/gity/internal/config"
	"github.com/DaiYuANg/gity/internal/platform/gitexec"
	"github.com/DaiYuANg/gity/internal/platform/gitrepo"
	platformstorage "github.com/DaiYuANg/gity/internal/platform/storage"
	"github.com/DaiYuANg/gity/internal/repository/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/repository/namespace"
	namespacememberrepo "github.com/DaiYuANg/gity/internal/repository/namespacemember"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	projectbranchprotectionrepo "github.com/DaiYuANg/gity/internal/repository/projectbranchprotection"
	projectpackagerepo "github.com/DaiYuANg/gity/internal/repository/projectpackage"
	projectpackagefilerepo "github.com/DaiYuANg/gity/internal/repository/projectpackagefile"
	projectpackageversionrepo "github.com/DaiYuANg/gity/internal/repository/projectpackageversion"
	userrepo "github.com/DaiYuANg/gity/internal/repository/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/repository/usertoken"
	namespaceservice "github.com/DaiYuANg/gity/internal/service/namespace"
	projectservice "github.com/DaiYuANg/gity/internal/service/project"
	userservice "github.com/DaiYuANg/gity/internal/service/user"

	"github.com/arcgolabs/dbx"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestPackageRegistryFlow(t *testing.T) {
	t.Parallel()

	dbPath := filepath.Join(t.TempDir(), "gity-package-test.db")
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
	projectBranchProtectionRepository, _ := projectbranchprotectionrepo.NewRepository(db)
	packageRepository, _ := projectpackagerepo.NewRepository(db)
	versionRepository, _ := projectpackageversionrepo.NewRepository(db)
	fileRepository, _ := projectpackagefilerepo.NewRepository(db)
	userRepository, _ := userrepo.NewRepository(db)
	userTokenRepository, _ := usertokenrepo.NewRepository(db)

	repoRoot := filepath.Join(t.TempDir(), "repos")
	storageRoot := filepath.Join(t.TempDir(), "storage")
	runner := gitexec.NewRunner(config.Settings{Git: config.GitSettings{Bin: "git", RepoRoot: repoRoot}})
	gitRepository := gitrepo.NewService(config.Settings{Git: config.GitSettings{RepoRoot: repoRoot}})
	storage, err := platformstorage.NewService(config.Settings{Storage: config.StorageSettings{Driver: "local", Root: storageRoot}})
	if err != nil {
		t.Fatalf("new storage service: %v", err)
	}

	userSvc := userservice.NewService(logger, userRepository, userTokenRepository)
	namespaceSvc := namespaceservice.NewService(logger, namespaceRepository, namespaceMemberRepository, userRepository)
	projectSvc := projectservice.NewService(logger, projectRepository, runner, gitRepository, namespaceRepository, projectBranchProtectionRepository)
	packageSvc := NewService(projectRepository, packageRepository, versionRepository, fileRepository, storage)

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
	if err := pushFixtureCommit(ctx, repoRoot, project.FullPath+".git"); err != nil {
		t.Fatalf("push fixture commit: %v", err)
	}

	fileRecord, err := packageSvc.UploadFile(ctx, project.ID, UploadFileInput{
		Type:          "maven",
		Name:          "io.gity:gity-api",
		Version:       "1.0.0",
		FileName:      "gity-api-1.0.0.jar",
		FilePath:      "io/gity/gity-api/1.0.0/gity-api-1.0.0.jar",
		ContentType:   "application/java-archive",
		ContentBase64: base64.StdEncoding.EncodeToString([]byte("jar-binary")),
	})
	if err != nil {
		t.Fatalf("upload package file: %v", err)
	}
	if fileRecord.ByteSize != int64(len("jar-binary")) {
		t.Fatalf("unexpected package file size: %d", fileRecord.ByteSize)
	}

	packages, err := packageSvc.ListPackages(ctx, project.ID)
	if err != nil {
		t.Fatalf("list packages: %v", err)
	}
	if len(packages) != 1 || packages[0].Type != "maven" {
		t.Fatalf("unexpected packages: %+v", packages)
	}

	detail, err := packageSvc.GetPackage(ctx, project.ID, packages[0].ID)
	if err != nil {
		t.Fatalf("get package detail: %v", err)
	}
	if len(detail.Versions) != 1 || len(detail.Versions[0].Files) != 1 {
		t.Fatalf("unexpected package detail: %+v", detail)
	}

	content, err := packageSvc.GetFileContent(ctx, project.ID, fileRecord.ID)
	if err != nil {
		t.Fatalf("get package file content: %v", err)
	}
	decoded, err := base64.StdEncoding.DecodeString(content.Content)
	if err != nil {
		t.Fatalf("decode package file content: %v", err)
	}
	if string(decoded) != "jar-binary" {
		t.Fatalf("unexpected package content: %s", string(decoded))
	}
}

func pushFixtureCommit(ctx context.Context, repoRoot string, repoPath string) error {
	worktree := filepath.Join(filepath.Dir(repoRoot), "fixture-worktree-package")
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
