package lfs_test

import (
	"context"
	"fmt"
	"log/slog"
	"path/filepath"
	"testing"

	lfsservice "github.com/DaiYuANg/gity/internal/application/lfs"
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
	projectlfslockrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_lfs_lock"
	projectlfsobjectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_lfs_object"
	userrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user_token"
	infrastorage "github.com/DaiYuANg/gity/internal/infrastructure/storage"
	"github.com/DaiYuANg/gity/internal/testutil"

	"github.com/arcgolabs/dbx"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestLFSFlow(t *testing.T) {
	t.Parallel()

	fixture := newLFSFixture(t)
	assertLFSObjectFlow(t, fixture)
	assertLFSLockFlow(t, fixture)
}

type lfsFixture struct {
	ctx             context.Context
	projectID       int64
	projectFullPath string
	ownerID         int64
	otherID         int64
	service         *lfsservice.Service
}

func newLFSFixture(t *testing.T) lfsFixture {
	t.Helper()

	dbPath := filepath.Join(t.TempDir(), "gity-lfs-test.db")
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
	projectLFSObjectRepository := testutil.Must(projectlfsobjectrepo.NewRepository(db))
	projectLFSLockRepository := testutil.Must(projectlfslockrepo.NewRepository(db))
	userRepository := testutil.Must(userrepo.NewRepository(db))
	userTokenRepository := testutil.Must(usertokenrepo.NewRepository(db))

	repoRoot := filepath.Join(t.TempDir(), "repos")
	storageRoot := filepath.Join(t.TempDir(), "storage")
	runner := gitexec.NewRunner(config.Settings{Git: config.GitSettings{Bin: "git", RepoRoot: repoRoot}})
	gitRepository := gitrepo.NewService(config.Settings{Git: config.GitSettings{RepoRoot: repoRoot}})
	storage := testutil.Must(infrastorage.NewService(config.Settings{Storage: config.StorageSettings{Driver: "local", Root: storageRoot}}))

	userSvc := userservice.NewService(logger, userRepository, userTokenRepository)
	organizationSvc := organizationservice.NewService(logger, organizationRepository, organizationMemberRepository, userRepository)
	projectSvc := projectservice.NewService(logger, projectRepository, runner, gitRepository, organizationRepository, projectBranchProtectionRepository)
	lfsSvc := lfsservice.NewService(projectRepository, projectLFSObjectRepository, projectLFSLockRepository, userRepository, storage)

	owner := testutil.Must(userSvc.Create(ctx, userservice.CreateInput{Username: "alice", DisplayName: "Alice", Email: "alice@gity.dev"}))
	other := testutil.Must(userSvc.Create(ctx, userservice.CreateInput{Username: "bob", DisplayName: "Bob", Email: "bob@gity.dev"}))
	space := testutil.Must(organizationSvc.Create(ctx, organizationservice.CreateInput{Name: "Core Team", PathKey: "core-team", OwnerUserID: owner.ID}))
	project := testutil.Must(projectSvc.Create(ctx, projectservice.CreateInput{OrganizationID: space.ID, Name: "Gity", PathKey: "gity", DefaultBranch: "main", Visibility: "private"}))

	return lfsFixture{
		ctx:             ctx,
		projectID:       project.ID,
		projectFullPath: project.FullPath,
		ownerID:         owner.ID,
		otherID:         other.ID,
		service:         lfsSvc,
	}
}

func assertLFSObjectFlow(t *testing.T, fixture lfsFixture) {
	t.Helper()

	oid := "1111111111111111111111111111111111111111111111111111111111111111"
	batchUpload := testutil.Must(fixture.service.PrepareBatch(fixture.ctx, fixture.projectID, lfsservice.BatchRequest{
		Operation: "upload",
		Objects:   []lfsservice.BatchObjectRequest{{OID: oid, Size: int64(len("hello-lfs"))}},
	}, "http://localhost:3000", fixture.projectFullPath+".git"))
	assertLFSBatchAction(t, batchUpload, "upload")

	uploaded := testutil.Must(fixture.service.UploadObject(fixture.ctx, fixture.projectID, oid, []byte("hello-lfs")))
	if uploaded.ByteSize != int64(len("hello-lfs")) {
		t.Fatalf("unexpected lfs object size: %d", uploaded.ByteSize)
	}

	batchDownload := testutil.Must(fixture.service.PrepareBatch(fixture.ctx, fixture.projectID, lfsservice.BatchRequest{
		Operation: "download",
		Objects:   []lfsservice.BatchObjectRequest{{OID: oid}},
	}, "http://localhost:3000", fixture.projectFullPath+".git"))
	assertLFSBatchAction(t, batchDownload, "download")

	downloaded := testutil.Must(fixture.service.DownloadObject(fixture.ctx, fixture.projectID, oid))
	if string(downloaded.Content) != "hello-lfs" {
		t.Fatalf("unexpected lfs content: %s", string(downloaded.Content))
	}
}

func assertLFSBatchAction(t *testing.T, response lfsservice.BatchResponse, actionName string) {
	t.Helper()
	if len(response.Objects) != 1 {
		t.Fatalf("unexpected lfs batch response: %+v", response)
	}
	if response.Objects[0].Actions[actionName].Href == "" {
		t.Fatalf("missing %s action href in lfs batch response: %+v", actionName, response)
	}
}

func assertLFSLockFlow(t *testing.T, fixture lfsFixture) {
	t.Helper()

	createdLock := testutil.Must(fixture.service.CreateLock(fixture.ctx, fixture.projectID, fixture.ownerID, lfsservice.CreateLockInput{Path: "assets/big.bin"}))
	if createdLock.Lock.ID == "" || createdLock.Lock.Owner.Name != "Alice" {
		t.Fatalf("unexpected created lock: %+v", createdLock)
	}

	if _, createLockErr := fixture.service.CreateLock(fixture.ctx, fixture.projectID, fixture.otherID, lfsservice.CreateLockInput{Path: "assets/big.bin"}); createLockErr == nil {
		t.Fatalf("expected duplicate lock create to fail")
	}

	createdOtherLock := testutil.Must(fixture.service.CreateLock(fixture.ctx, fixture.projectID, fixture.otherID, lfsservice.CreateLockInput{Path: "assets/other.bin"}))
	assertLFSLockLists(t, fixture)
	assertLFSUnlock(t, fixture, createdOtherLock.Lock.ID)
}

func assertLFSLockLists(t *testing.T, fixture lfsFixture) {
	t.Helper()

	listed := testutil.Must(fixture.service.ListLocks(fixture.ctx, fixture.projectID, lfsservice.LockListInput{Limit: 10}))
	if len(listed.Locks) != 2 {
		t.Fatalf("expected 2 lfs locks, got %d", len(listed.Locks))
	}

	verified := testutil.Must(fixture.service.VerifyLocks(fixture.ctx, fixture.projectID, fixture.ownerID, lfsservice.LockListInput{Limit: 10}))
	if len(verified.Ours) != 1 || len(verified.Theirs) != 1 {
		t.Fatalf("unexpected lfs verify result: %+v", verified)
	}
}

func assertLFSUnlock(t *testing.T, fixture lfsFixture, lockID string) {
	t.Helper()

	if _, unlockErr := fixture.service.Unlock(fixture.ctx, fixture.projectID, fixture.ownerID, lockID, lfsservice.UnlockInput{}); unlockErr == nil {
		t.Fatalf("expected unlocking another user's lock without force to fail")
	}

	unlocked := testutil.Must(fixture.service.Unlock(fixture.ctx, fixture.projectID, fixture.ownerID, lockID, lfsservice.UnlockInput{Force: true}))
	if unlocked.Lock.ID != lockID {
		t.Fatalf("unexpected unlocked lfs lock: %+v", unlocked)
	}
}
