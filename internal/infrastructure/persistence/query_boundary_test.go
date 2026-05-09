package persistence_test

import (
	"context"
	"fmt"
	"path/filepath"
	"testing"

	"github.com/DaiYuANg/gity/internal/infrastructure/persistence/core"
	organizationrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/organization"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectissuerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_issue"
	projectlfslockrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_lfs_lock"
	projectlfsobjectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_lfs_object"
	projectmergerequestrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_merge_request"
	projectpackagerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_package"
	projectpackageversionrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_package_version"
	userrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user"
	"github.com/DaiYuANg/gity/internal/testutil"

	"github.com/arcgolabs/dbx"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestProjectScopedRepositoryQueriesDoNotLeakAcrossProjects(t *testing.T) {
	t.Parallel()

	fixture := newBoundaryFixture(t)
	assertIssueQueryBoundary(t, fixture)
	assertMergeRequestQueryBoundary(t, fixture)
	assertPackageQueryBoundary(t, fixture)
	assertLFSObjectQueryBoundary(t, fixture)
	assertLFSLockQueryBoundary(t, fixture)
}

type boundaryFixture struct {
	ctx             context.Context
	ownerID         int64
	firstProjectID  int64
	secondProjectID int64
	issues          *projectissuerepo.Repository
	mergeRequests   *projectmergerequestrepo.Repository
	packages        *projectpackagerepo.Repository
	versions        *projectpackageversionrepo.Repository
	lfsObjects      *projectlfsobjectrepo.Repository
	lfsLocks        *projectlfslockrepo.Repository
}

func newBoundaryFixture(t *testing.T) boundaryFixture {
	t.Helper()

	ctx := context.Background()
	db := openBoundaryTestDB(t)
	testutil.CleanupClose(t, "db", db)
	testutil.RequireNoError(t, core.EnsureSchema(ctx, db), "ensure schema")

	organizations := testutil.Must(organizationrepo.NewRepository(db))
	projects := testutil.Must(projectrepo.NewRepository(db))
	users := testutil.Must(userrepo.NewRepository(db))
	issues := testutil.Must(projectissuerepo.NewRepository(db))
	mergeRequests := testutil.Must(projectmergerequestrepo.NewRepository(db))
	packages := testutil.Must(projectpackagerepo.NewRepository(db))
	versions := testutil.Must(projectpackageversionrepo.NewRepository(db))
	lfsObjects := testutil.Must(projectlfsobjectrepo.NewRepository(db))
	lfsLocks := testutil.Must(projectlfslockrepo.NewRepository(db))

	owner := testutil.Must(users.Create(ctx, userrepo.CreateInput{Username: "alice", DisplayName: "Alice", Email: "alice@gity.dev"}))
	organization := testutil.Must(organizations.Create(ctx, organizationrepo.CreateInput{Name: "Core Team", PathKey: "core-team"}))
	firstProject := testutil.Must(projects.Create(ctx, projectrepo.CreateInput{OrganizationID: organization.ID, Name: "First", PathKey: "first", Visibility: "private"}, organization))
	secondProject := testutil.Must(projects.Create(ctx, projectrepo.CreateInput{OrganizationID: organization.ID, Name: "Second", PathKey: "second", Visibility: "private"}, organization))

	return boundaryFixture{
		ctx:             ctx,
		ownerID:         owner.ID,
		firstProjectID:  firstProject.ID,
		secondProjectID: secondProject.ID,
		issues:          issues,
		mergeRequests:   mergeRequests,
		packages:        packages,
		versions:        versions,
		lfsObjects:      lfsObjects,
		lfsLocks:        lfsLocks,
	}
}

func assertIssueQueryBoundary(t *testing.T, fixture boundaryFixture) {
	t.Helper()

	firstIssue := testutil.Must(fixture.issues.Create(fixture.ctx, projectissuerepo.CreateInput{ProjectID: fixture.firstProjectID, AuthorUserID: fixture.ownerID, Title: "first issue"}))
	secondIssue := testutil.Must(fixture.issues.Create(fixture.ctx, projectissuerepo.CreateInput{ProjectID: fixture.secondProjectID, AuthorUserID: fixture.ownerID, Title: "second issue"}))
	foundIssue := testutil.Must(fixture.issues.GetByProjectAndIID(fixture.ctx, fixture.secondProjectID, secondIssue.IID))
	if foundIssue.ID != secondIssue.ID || foundIssue.ID == firstIssue.ID {
		t.Fatalf("issue query leaked across projects: got %+v, want %+v", foundIssue, secondIssue)
	}
}

func assertMergeRequestQueryBoundary(t *testing.T, fixture boundaryFixture) {
	t.Helper()

	firstMR := testutil.Must(fixture.mergeRequests.Create(fixture.ctx, projectmergerequestrepo.CreateInput{ProjectID: fixture.firstProjectID, AuthorUserID: fixture.ownerID, Title: "first mr", SourceBranch: "feature", TargetBranch: "main"}))
	secondMR := testutil.Must(fixture.mergeRequests.Create(fixture.ctx, projectmergerequestrepo.CreateInput{ProjectID: fixture.secondProjectID, AuthorUserID: fixture.ownerID, Title: "second mr", SourceBranch: "feature", TargetBranch: "main"}))
	foundMR := testutil.Must(fixture.mergeRequests.GetByProjectAndIID(fixture.ctx, fixture.secondProjectID, secondMR.IID))
	if foundMR.ID != secondMR.ID || foundMR.ID == firstMR.ID {
		t.Fatalf("merge request query leaked across projects: got %+v, want %+v", foundMR, secondMR)
	}
}

func assertPackageQueryBoundary(t *testing.T, fixture boundaryFixture) {
	t.Helper()

	firstPackage := testutil.Must(fixture.packages.Create(fixture.ctx, projectpackagerepo.CreateInput{ProjectID: fixture.firstProjectID, Type: "maven", Name: "io.gity:gity-api"}))
	secondPackage := testutil.Must(fixture.packages.Create(fixture.ctx, projectpackagerepo.CreateInput{ProjectID: fixture.secondProjectID, Type: "maven", Name: "io.gity:gity-api"}))
	foundPackage := testutil.Must(fixture.packages.GetByProjectTypeAndName(fixture.ctx, fixture.secondProjectID, "maven", "io.gity:gity-api"))
	if foundPackage.ID != secondPackage.ID || foundPackage.ID == firstPackage.ID {
		t.Fatalf("package query leaked across projects: got %+v, want %+v", foundPackage, secondPackage)
	}
	assertPackageVersionQueryBoundary(t, fixture, firstPackage.ID, secondPackage.ID)
}

func assertPackageVersionQueryBoundary(t *testing.T, fixture boundaryFixture, firstPackageID, secondPackageID int64) {
	t.Helper()

	firstVersion := testutil.Must(fixture.versions.Create(fixture.ctx, projectpackageversionrepo.CreateInput{ProjectPackageID: firstPackageID, Version: "1.0.0"}))
	secondVersion := testutil.Must(fixture.versions.Create(fixture.ctx, projectpackageversionrepo.CreateInput{ProjectPackageID: secondPackageID, Version: "1.0.0"}))
	foundVersion := testutil.Must(fixture.versions.GetByPackageAndVersion(fixture.ctx, secondPackageID, "1.0.0"))
	if foundVersion.ID != secondVersion.ID || foundVersion.ID == firstVersion.ID {
		t.Fatalf("package version query leaked across packages: got %+v, want %+v", foundVersion, secondVersion)
	}
}

func assertLFSObjectQueryBoundary(t *testing.T, fixture boundaryFixture) {
	t.Helper()

	oid := "2222222222222222222222222222222222222222222222222222222222222222"
	firstObject := testutil.Must(fixture.lfsObjects.Create(fixture.ctx, fixture.firstProjectID, oid, 1, "lfs/first"))
	secondObject := testutil.Must(fixture.lfsObjects.Create(fixture.ctx, fixture.secondProjectID, oid, 2, "lfs/second"))
	foundObject := testutil.Must(fixture.lfsObjects.GetByProjectAndOID(fixture.ctx, fixture.secondProjectID, oid))
	if foundObject.ID != secondObject.ID || foundObject.ID == firstObject.ID {
		t.Fatalf("lfs object query leaked across projects: got %+v, want %+v", foundObject, secondObject)
	}
}

func assertLFSLockQueryBoundary(t *testing.T, fixture boundaryFixture) {
	t.Helper()

	_ = testutil.Must(fixture.lfsLocks.Create(fixture.ctx, projectlfslockrepo.CreateInput{ProjectID: fixture.firstProjectID, OwnerUserID: fixture.ownerID, Path: "assets/big.bin"}))
	secondLock := testutil.Must(fixture.lfsLocks.Create(fixture.ctx, projectlfslockrepo.CreateInput{ProjectID: fixture.secondProjectID, OwnerUserID: fixture.ownerID, Path: "assets/big.bin"}))
	foundLock := testutil.Must(fixture.lfsLocks.GetByProjectAndPath(fixture.ctx, fixture.secondProjectID, "assets/big.bin"))
	if foundLock.ID != secondLock.ID {
		t.Fatalf("lfs lock path query leaked across projects: got %+v, want %+v", foundLock, secondLock)
	}
	listedLocks := testutil.Must(fixture.lfsLocks.ListByProjectID(fixture.ctx, projectlfslockrepo.ListInput{ProjectID: fixture.secondProjectID, Path: "assets/big.bin", Limit: 10}))
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
