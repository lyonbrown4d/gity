package repository_test

import (
	"context"
	"fmt"
	"path/filepath"
	"testing"

	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
	"github.com/DaiYuANg/gity/internal/repository/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/repository/namespace"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	projectissuerepo "github.com/DaiYuANg/gity/internal/repository/projectissue"
	projectlfslockrepo "github.com/DaiYuANg/gity/internal/repository/projectlfslock"
	projectlfsobjectrepo "github.com/DaiYuANg/gity/internal/repository/projectlfsobject"
	projectmergerequestrepo "github.com/DaiYuANg/gity/internal/repository/projectmergerequest"
	projectpackagerepo "github.com/DaiYuANg/gity/internal/repository/projectpackage"
	projectpackageversionrepo "github.com/DaiYuANg/gity/internal/repository/projectpackageversion"
	userrepo "github.com/DaiYuANg/gity/internal/repository/user"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestProjectScopedRepositoryQueriesDoNotLeakAcrossProjects(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	db := openBoundaryTestDB(t)
	defer db.Close()
	if err := core.EnsureSchema(ctx, db); err != nil {
		t.Fatalf("ensure schema: %v", err)
	}

	namespaces, _ := namespacerepo.NewRepository(db)
	projects, _ := projectrepo.NewRepository(db)
	users, _ := userrepo.NewRepository(db)
	issues, _ := projectissuerepo.NewRepository(db)
	mergeRequests, _ := projectmergerequestrepo.NewRepository(db)
	packages, _ := projectpackagerepo.NewRepository(db)
	versions, _ := projectpackageversionrepo.NewRepository(db)
	lfsObjects, _ := projectlfsobjectrepo.NewRepository(db)
	lfsLocks, _ := projectlfslockrepo.NewRepository(db)

	owner, err := users.Create(ctx, userrepo.CreateInput{Username: "alice", DisplayName: "Alice", Email: "alice@gity.dev"})
	if err != nil {
		t.Fatalf("create user: %v", err)
	}
	namespace, err := namespaces.Create(ctx, namespacerepo.CreateInput{Kind: "group", Name: "Core Team", PathKey: "core-team"})
	if err != nil {
		t.Fatalf("create namespace: %v", err)
	}
	firstProject, err := projects.Create(ctx, projectrepo.CreateInput{NamespaceID: namespace.ID, Name: "First", PathKey: "first", Visibility: "private"}, namespace)
	if err != nil {
		t.Fatalf("create first project: %v", err)
	}
	secondProject, err := projects.Create(ctx, projectrepo.CreateInput{NamespaceID: namespace.ID, Name: "Second", PathKey: "second", Visibility: "private"}, namespace)
	if err != nil {
		t.Fatalf("create second project: %v", err)
	}

	firstIssue, err := issues.Create(ctx, projectissuerepo.CreateInput{ProjectID: firstProject.ID, AuthorUserID: owner.ID, Title: "first issue"})
	if err != nil {
		t.Fatalf("create first issue: %v", err)
	}
	secondIssue, err := issues.Create(ctx, projectissuerepo.CreateInput{ProjectID: secondProject.ID, AuthorUserID: owner.ID, Title: "second issue"})
	if err != nil {
		t.Fatalf("create second issue: %v", err)
	}
	foundIssue, err := issues.GetByProjectAndIID(ctx, secondProject.ID, secondIssue.IID)
	if err != nil {
		t.Fatalf("get scoped issue: %v", err)
	}
	if foundIssue.ID != secondIssue.ID || foundIssue.ID == firstIssue.ID {
		t.Fatalf("issue query leaked across projects: got %+v, want %+v", foundIssue, secondIssue)
	}

	firstMR, err := mergeRequests.Create(ctx, projectmergerequestrepo.CreateInput{ProjectID: firstProject.ID, AuthorUserID: owner.ID, Title: "first mr", SourceBranch: "feature", TargetBranch: "main"})
	if err != nil {
		t.Fatalf("create first merge request: %v", err)
	}
	secondMR, err := mergeRequests.Create(ctx, projectmergerequestrepo.CreateInput{ProjectID: secondProject.ID, AuthorUserID: owner.ID, Title: "second mr", SourceBranch: "feature", TargetBranch: "main"})
	if err != nil {
		t.Fatalf("create second merge request: %v", err)
	}
	foundMR, err := mergeRequests.GetByProjectAndIID(ctx, secondProject.ID, secondMR.IID)
	if err != nil {
		t.Fatalf("get scoped merge request: %v", err)
	}
	if foundMR.ID != secondMR.ID || foundMR.ID == firstMR.ID {
		t.Fatalf("merge request query leaked across projects: got %+v, want %+v", foundMR, secondMR)
	}

	firstPackage, err := packages.Create(ctx, projectpackagerepo.CreateInput{ProjectID: firstProject.ID, Type: "maven", Name: "io.gity:gity-api"})
	if err != nil {
		t.Fatalf("create first package: %v", err)
	}
	secondPackage, err := packages.Create(ctx, projectpackagerepo.CreateInput{ProjectID: secondProject.ID, Type: "maven", Name: "io.gity:gity-api"})
	if err != nil {
		t.Fatalf("create second package: %v", err)
	}
	foundPackage, err := packages.GetByProjectTypeAndName(ctx, secondProject.ID, "maven", "io.gity:gity-api")
	if err != nil {
		t.Fatalf("get scoped package: %v", err)
	}
	if foundPackage.ID != secondPackage.ID || foundPackage.ID == firstPackage.ID {
		t.Fatalf("package query leaked across projects: got %+v, want %+v", foundPackage, secondPackage)
	}

	firstVersion, err := versions.Create(ctx, projectpackageversionrepo.CreateInput{ProjectPackageID: firstPackage.ID, Version: "1.0.0"})
	if err != nil {
		t.Fatalf("create first package version: %v", err)
	}
	secondVersion, err := versions.Create(ctx, projectpackageversionrepo.CreateInput{ProjectPackageID: secondPackage.ID, Version: "1.0.0"})
	if err != nil {
		t.Fatalf("create second package version: %v", err)
	}
	foundVersion, err := versions.GetByPackageAndVersion(ctx, secondPackage.ID, "1.0.0")
	if err != nil {
		t.Fatalf("get scoped package version: %v", err)
	}
	if foundVersion.ID != secondVersion.ID || foundVersion.ID == firstVersion.ID {
		t.Fatalf("package version query leaked across packages: got %+v, want %+v", foundVersion, secondVersion)
	}

	oid := "2222222222222222222222222222222222222222222222222222222222222222"
	firstObject, err := lfsObjects.Create(ctx, firstProject.ID, oid, 1, "lfs/first")
	if err != nil {
		t.Fatalf("create first lfs object: %v", err)
	}
	secondObject, err := lfsObjects.Create(ctx, secondProject.ID, oid, 2, "lfs/second")
	if err != nil {
		t.Fatalf("create second lfs object: %v", err)
	}
	foundObject, err := lfsObjects.GetByProjectAndOID(ctx, secondProject.ID, oid)
	if err != nil {
		t.Fatalf("get scoped lfs object: %v", err)
	}
	if foundObject.ID != secondObject.ID || foundObject.ID == firstObject.ID {
		t.Fatalf("lfs object query leaked across projects: got %+v, want %+v", foundObject, secondObject)
	}

	_, err = lfsLocks.Create(ctx, projectlfslockrepo.CreateInput{ProjectID: firstProject.ID, OwnerUserID: owner.ID, Path: "assets/big.bin"})
	if err != nil {
		t.Fatalf("create first lfs lock: %v", err)
	}
	secondLock, err := lfsLocks.Create(ctx, projectlfslockrepo.CreateInput{ProjectID: secondProject.ID, OwnerUserID: owner.ID, Path: "assets/big.bin"})
	if err != nil {
		t.Fatalf("create second lfs lock: %v", err)
	}
	foundLock, err := lfsLocks.GetByProjectAndPath(ctx, secondProject.ID, "assets/big.bin")
	if err != nil {
		t.Fatalf("get scoped lfs lock: %v", err)
	}
	if foundLock.ID != secondLock.ID {
		t.Fatalf("lfs lock path query leaked across projects: got %+v, want %+v", foundLock, secondLock)
	}
	listedLocks, err := lfsLocks.ListByProjectID(ctx, projectlfslockrepo.ListInput{ProjectID: secondProject.ID, Path: "assets/big.bin", Limit: 10})
	if err != nil {
		t.Fatalf("list scoped lfs locks: %v", err)
	}
	if listedLocks.Len() != 1 || listedLocks.Values()[0].ID != secondLock.ID {
		t.Fatalf("lfs lock list leaked across projects: %+v", listedLocks.Values())
	}
}

func openBoundaryTestDB(t *testing.T) *dbx.DB {
	t.Helper()
	dbPath := filepath.Join(t.TempDir(), "gity-query-boundary-test.db")
	db, err := dbx.Open(
		dbx.WithDriver("sqlite"),
		dbx.WithDSN(fmt.Sprintf("file:%s?_pragma=foreign_keys(1)", dbPath)),
		dbx.WithDialect(sqliteDialect.New()),
	)
	if err != nil {
		t.Fatalf("open db: %v", err)
	}
	return db
}
