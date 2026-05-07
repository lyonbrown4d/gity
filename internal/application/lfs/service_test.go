package lfs

import (
	"context"
	"fmt"
	"log/slog"
	"path/filepath"
	"testing"

	namespaceservice "github.com/DaiYuANg/gity/internal/application/namespace"
	projectservice "github.com/DaiYuANg/gity/internal/application/project"
	userservice "github.com/DaiYuANg/gity/internal/application/user"
	"github.com/DaiYuANg/gity/internal/config"
	"github.com/DaiYuANg/gity/internal/infrastructure/persistence/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespace"
	namespacememberrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespacemember"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectbranchprotectionrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectbranchprotection"
	projectlfslockrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectlfslock"
	projectlfsobjectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectlfsobject"
	userrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/usertoken"
	"github.com/DaiYuANg/gity/internal/platform/gitexec"
	"github.com/DaiYuANg/gity/internal/platform/gitrepo"
	platformstorage "github.com/DaiYuANg/gity/internal/platform/storage"

	"github.com/arcgolabs/dbx"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestLFSFlow(t *testing.T) {
	t.Parallel()

	dbPath := filepath.Join(t.TempDir(), "gity-lfs-test.db")
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
	projectLFSObjectRepository, _ := projectlfsobjectrepo.NewRepository(db)
	projectLFSLockRepository, _ := projectlfslockrepo.NewRepository(db)
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
	lfsSvc := NewService(projectRepository, projectLFSObjectRepository, projectLFSLockRepository, userRepository, storage)

	owner, err := userSvc.Create(ctx, userservice.CreateInput{Username: "alice", DisplayName: "Alice", Email: "alice@gity.dev"})
	if err != nil {
		t.Fatalf("create owner user: %v", err)
	}
	other, err := userSvc.Create(ctx, userservice.CreateInput{Username: "bob", DisplayName: "Bob", Email: "bob@gity.dev"})
	if err != nil {
		t.Fatalf("create other user: %v", err)
	}
	space, err := namespaceSvc.Create(ctx, namespaceservice.CreateInput{Kind: "group", Name: "Core Team", PathKey: "core-team", OwnerUserID: owner.ID})
	if err != nil {
		t.Fatalf("create namespace: %v", err)
	}
	project, err := projectSvc.Create(ctx, projectservice.CreateInput{NamespaceID: space.ID, Name: "Gity", PathKey: "gity", DefaultBranch: "main", Visibility: "private"})
	if err != nil {
		t.Fatalf("create project: %v", err)
	}

	oid := "1111111111111111111111111111111111111111111111111111111111111111"
	batchUpload, err := lfsSvc.PrepareBatch(ctx, project.ID, BatchRequest{
		Operation: "upload",
		Objects:   []BatchObjectRequest{{OID: oid, Size: int64(len("hello-lfs"))}},
	}, "http://localhost:3000", project.FullPath+".git")
	if err != nil {
		t.Fatalf("prepare upload batch: %v", err)
	}
	if len(batchUpload.Objects) != 1 || batchUpload.Objects[0].Actions["upload"].Href == "" {
		t.Fatalf("unexpected upload batch response: %+v", batchUpload)
	}

	uploaded, err := lfsSvc.UploadObject(ctx, project.ID, oid, []byte("hello-lfs"))
	if err != nil {
		t.Fatalf("upload lfs object: %v", err)
	}
	if uploaded.ByteSize != int64(len("hello-lfs")) {
		t.Fatalf("unexpected lfs object size: %d", uploaded.ByteSize)
	}

	batchDownload, err := lfsSvc.PrepareBatch(ctx, project.ID, BatchRequest{
		Operation: "download",
		Objects:   []BatchObjectRequest{{OID: oid}},
	}, "http://localhost:3000", project.FullPath+".git")
	if err != nil {
		t.Fatalf("prepare download batch: %v", err)
	}
	if len(batchDownload.Objects) != 1 || batchDownload.Objects[0].Actions["download"].Href == "" {
		t.Fatalf("unexpected download batch response: %+v", batchDownload)
	}

	downloaded, err := lfsSvc.DownloadObject(ctx, project.ID, oid)
	if err != nil {
		t.Fatalf("download lfs object: %v", err)
	}
	if string(downloaded.Content) != "hello-lfs" {
		t.Fatalf("unexpected lfs content: %s", string(downloaded.Content))
	}

	createdLock, err := lfsSvc.CreateLock(ctx, project.ID, owner.ID, CreateLockInput{Path: "assets/big.bin"})
	if err != nil {
		t.Fatalf("create lfs lock: %v", err)
	}
	if createdLock.Lock.ID == "" || createdLock.Lock.Owner.Name != "Alice" {
		t.Fatalf("unexpected created lock: %+v", createdLock)
	}

	if _, err := lfsSvc.CreateLock(ctx, project.ID, other.ID, CreateLockInput{Path: "assets/big.bin"}); err == nil {
		t.Fatalf("expected duplicate lock create to fail")
	}

	createdOtherLock, err := lfsSvc.CreateLock(ctx, project.ID, other.ID, CreateLockInput{Path: "assets/other.bin"})
	if err != nil {
		t.Fatalf("create second lfs lock: %v", err)
	}

	listed, err := lfsSvc.ListLocks(ctx, project.ID, LockListInput{Limit: 10})
	if err != nil {
		t.Fatalf("list lfs locks: %v", err)
	}
	if len(listed.Locks) != 2 {
		t.Fatalf("expected 2 lfs locks, got %d", len(listed.Locks))
	}

	verified, err := lfsSvc.VerifyLocks(ctx, project.ID, owner.ID, LockListInput{Limit: 10})
	if err != nil {
		t.Fatalf("verify lfs locks: %v", err)
	}
	if len(verified.Ours) != 1 || len(verified.Theirs) != 1 {
		t.Fatalf("unexpected lfs verify result: %+v", verified)
	}

	if _, err := lfsSvc.Unlock(ctx, project.ID, owner.ID, createdOtherLock.Lock.ID, UnlockInput{}); err == nil {
		t.Fatalf("expected unlocking another user's lock without force to fail")
	}

	unlocked, err := lfsSvc.Unlock(ctx, project.ID, owner.ID, createdOtherLock.Lock.ID, UnlockInput{Force: true})
	if err != nil {
		t.Fatalf("force unlock lfs lock: %v", err)
	}
	if unlocked.Lock.ID != createdOtherLock.Lock.ID {
		t.Fatalf("unexpected unlocked lfs lock: %+v", unlocked)
	}
}
