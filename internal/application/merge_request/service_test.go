package mergerequest_test

import (
	"context"
	"fmt"
	"log/slog"
	"path/filepath"
	"strings"
	"testing"

	jobservice "github.com/DaiYuANg/gity/internal/application/job"
	mergerequestservice "github.com/DaiYuANg/gity/internal/application/merge_request"
	organizationservice "github.com/DaiYuANg/gity/internal/application/organization"
	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
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
	projectjobrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_job"
	projectmergerequestrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_merge_request"
	projectpipelinerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_pipeline"
	projectpipelinejobrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_pipeline_job"
	userrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/user_token"
	"github.com/DaiYuANg/gity/internal/testutil"

	"github.com/arcgolabs/dbx"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestMergeRequestFlow(t *testing.T) {
	t.Parallel()

	fixture := newMergeRequestFixture(t, false)
	mrIID := assertCreateMergeRequest(t, fixture, "merge feature")
	assertMergeRequestDiff(t, fixture, mrIID)
	assertMergeRequestMerge(t, fixture, mrIID)
	assertCloseSecondMergeRequest(t, fixture)
	assertListMergeRequests(t, fixture)
}

func TestMergeRequestMergeRequiresSuccessfulPipelineWhenCIConfigExists(t *testing.T) {
	t.Parallel()

	fixture := newMergeRequestFixture(t, true)
	sourceBranch := addMergeRequestCIConfig(t, fixture)
	mrIID := assertCreateMergeRequest(t, fixture, "merge feature with checks")
	assertMissingMergeRequestChecks(t, fixture, mrIID)
	assertMergeRequestBlocked(t, fixture, mrIID)
	createFailedPipelineAndMarkSucceeded(t, fixture, sourceBranch.Hash, mrIID)
	assertMergeAfterSuccessfulPipeline(t, fixture, mrIID)
	assertTargetBranchPipeline(t, fixture)
}

type mergeRequestFixture struct {
	ctx                    context.Context
	repoRoot               string
	projectID              int64
	projectFullPath        string
	projectDefaultBranch   string
	ownerID                int64
	runner                 *gitexec.Runner
	gitRepository          *gitrepo.Service
	mergeRequestService    *mergerequestservice.Service
	pipelineRepository     *projectpipelinerepo.Repository
	projectPipelineService *pipelineservice.Service
}

func newMergeRequestFixture(t *testing.T, withPipelineService bool) mergeRequestFixture {
	t.Helper()

	dbPath := filepath.Join(t.TempDir(), "gity-mr-checks-test.db")
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
	mergeRequestRepository := testutil.Must(projectmergerequestrepo.NewRepository(db))
	pipelineRepository := testutil.Must(projectpipelinerepo.NewRepository(db))
	pipelineJobRepository := createMRPipelineJobRepository(db, withPipelineService)
	jobRepository := createMRJobRepository(db, withPipelineService)
	userRepository := testutil.Must(userrepo.NewRepository(db))
	userTokenRepository := testutil.Must(usertokenrepo.NewRepository(db))

	repoRoot := filepath.Join(t.TempDir(), "repos")
	runner := gitexec.NewRunner(config.Settings{Git: config.GitSettings{Bin: "git", RepoRoot: repoRoot}})
	gitRepository := gitrepo.NewService(config.Settings{Git: config.GitSettings{RepoRoot: repoRoot}})

	userSvc := userservice.NewService(logger, userRepository, userTokenRepository)
	organizationSvc := organizationservice.NewService(logger, organizationRepository, organizationMemberRepository, userRepository)
	projectSvc := projectservice.NewService(logger, projectRepository, runner, gitRepository, organizationRepository, projectBranchProtectionRepository)
	pipelineSvc := createMRPipelineService(logger, projectRepository, pipelineRepository, pipelineJobRepository, jobRepository, gitRepository)
	mergeRequestSvc := mergerequestservice.NewService(projectRepository, mergeRequestRepository, userRepository, gitRepository, runner, mergerequestservice.NewPipelineDeps(pipelineRepository, pipelineSvc))

	owner := testutil.Must(userSvc.Create(ctx, userservice.CreateInput{Username: "alice", DisplayName: "Alice", Email: "alice@gity.dev"}))
	space := testutil.Must(organizationSvc.Create(ctx, organizationservice.CreateInput{Name: "Core Team", PathKey: "core-team", OwnerUserID: owner.ID}))
	project := testutil.Must(projectSvc.Create(ctx, projectservice.CreateInput{OrganizationID: space.ID, Name: "Gity", PathKey: "gity", DefaultBranch: "main", Visibility: "private"}))
	testutil.RequireNoError(t, pushFixtureBranches(ctx, repoRoot, project.FullPath+".git"), "push fixture branches")

	return mergeRequestFixture{
		ctx:                    ctx,
		repoRoot:               repoRoot,
		projectID:              project.ID,
		projectFullPath:        project.FullPath,
		projectDefaultBranch:   project.DefaultBranch,
		ownerID:                owner.ID,
		runner:                 runner,
		gitRepository:          gitRepository,
		mergeRequestService:    mergeRequestSvc,
		pipelineRepository:     pipelineRepository,
		projectPipelineService: pipelineSvc,
	}
}

func createMRPipelineJobRepository(db *dbx.DB, enabled bool) *projectpipelinejobrepo.Repository {
	if !enabled {
		return nil
	}
	return testutil.Must(projectpipelinejobrepo.NewRepository(db))
}

func createMRJobRepository(db *dbx.DB, enabled bool) *projectjobrepo.Repository {
	if !enabled {
		return nil
	}
	return testutil.Must(projectjobrepo.NewRepository(db))
}

func createMRPipelineService(logger *slog.Logger, projectRepository *projectrepo.Repository, pipelineRepository *projectpipelinerepo.Repository, pipelineJobRepository *projectpipelinejobrepo.Repository, jobRepository *projectjobrepo.Repository, gitRepository *gitrepo.Service) *pipelineservice.Service {
	if pipelineJobRepository == nil || jobRepository == nil {
		return nil
	}
	jobSvc := jobservice.NewService(logger, projectRepository, jobRepository, nil, nil, nil)
	return pipelineservice.NewService(projectRepository, pipelineRepository, pipelineJobRepository, jobSvc, jobRepository, gitRepository)
}

func assertCreateMergeRequest(t *testing.T, fixture mergeRequestFixture, title string) int64 {
	t.Helper()

	mr := testutil.Must(fixture.mergeRequestService.Create(fixture.ctx, fixture.projectID, mergerequestservice.CreateInput{AuthorUserID: fixture.ownerID, Title: title, Description: title, SourceBranch: "feature", TargetBranch: "main"}))
	if mr.IID == 0 {
		t.Fatalf("expected merge request iid to be assigned")
	}
	return mr.IID
}

func assertMergeRequestDiff(t *testing.T, fixture mergeRequestFixture, mrIID int64) {
	t.Helper()

	diff := testutil.Must(fixture.mergeRequestService.GetDiff(fixture.ctx, fixture.projectID, mrIID))
	if !strings.Contains(diff.Diff, "feature.txt") {
		t.Fatalf("expected diff to include feature file, got %s", diff.Diff)
	}
}

func assertMergeRequestMerge(t *testing.T, fixture mergeRequestFixture, mrIID int64) {
	t.Helper()

	merged := testutil.Must(fixture.mergeRequestService.Merge(fixture.ctx, fixture.projectID, mrIID, mergerequestservice.MergeInput{AuthorName: "Gity Test", AuthorEmail: "test@gity.dev"}))
	if merged.State != "merged" {
		t.Fatalf("expected merged state, got %s", merged.State)
	}
}

func assertCloseSecondMergeRequest(t *testing.T, fixture mergeRequestFixture) {
	t.Helper()

	closedMR := testutil.Must(fixture.mergeRequestService.Create(fixture.ctx, fixture.projectID, mergerequestservice.CreateInput{AuthorUserID: fixture.ownerID, Title: "close feature", Description: "close feature", SourceBranch: "feature", TargetBranch: "main"}))
	closedState := "closed"
	updated := testutil.Must(fixture.mergeRequestService.Update(fixture.ctx, fixture.projectID, closedMR.IID, mergerequestservice.UpdateInput{State: &closedState}))
	if updated.State != "closed" {
		t.Fatalf("unexpected merge request state: %s", updated.State)
	}
}

func assertListMergeRequests(t *testing.T, fixture mergeRequestFixture) {
	t.Helper()

	items := testutil.Must(fixture.mergeRequestService.List(fixture.ctx, fixture.projectID))
	if len(items) != 2 || items[0].SourceBranch != "feature" || items[0].TargetBranch != "main" {
		t.Fatalf("unexpected merge requests: %+v", items)
	}
}

func addMergeRequestCIConfig(t *testing.T, fixture mergeRequestFixture) gitrepo.Branch {
	t.Helper()

	testutil.RequireNoError(t, fixture.runner.CreateFileCommit(fixture.ctx, fixture.projectFullPath+".git", gitexec.CreateFileCommitInput{
		BranchName:  "feature",
		FilePath:    ".gity-ci.plano",
		Content:     mergeRequestCIConfig(),
		Message:     "Add CI config",
		AuthorName:  "Gity Test",
		AuthorEmail: "test@gity.dev",
	}), "add ci config")
	return findBranch(fixture.ctx, t, fixture.gitRepository, fixture.projectFullPath+".git", fixture.projectDefaultBranch, "feature")
}

func assertMissingMergeRequestChecks(t *testing.T, fixture mergeRequestFixture, mrIID int64) {
	t.Helper()

	checks := testutil.Must(fixture.mergeRequestService.GetChecks(fixture.ctx, fixture.projectID, mrIID))
	if !checks.Required || checks.Mergeable || checks.Status != "missing" {
		t.Fatalf("expected missing required checks: %+v", checks)
	}
}

func assertMergeRequestBlocked(t *testing.T, fixture mergeRequestFixture, mrIID int64) {
	t.Helper()

	if _, mergeErr := fixture.mergeRequestService.Merge(fixture.ctx, fixture.projectID, mrIID, mergerequestservice.MergeInput{AuthorName: "Gity Test", AuthorEmail: "test@gity.dev"}); mergeErr == nil {
		t.Fatalf("expected merge to be blocked before pipeline exists")
	}
}

func createFailedPipelineAndMarkSucceeded(t *testing.T, fixture mergeRequestFixture, sourceHash string, mrIID int64) {
	t.Helper()

	pipeline := testutil.Must(fixture.pipelineRepository.Create(fixture.ctx, projectpipelinerepo.CreateInput{
		ProjectID:     fixture.projectID,
		Name:          "merge-request",
		Source:        "push",
		RefName:       "feature",
		CommitSHA:     sourceHash,
		Status:        projectpipelinerepo.StatusFailed,
		ConfigSource:  ".gity-ci.plano",
		ConfigContent: mergeRequestCIConfig(),
	}))
	checks := testutil.Must(fixture.mergeRequestService.GetChecks(fixture.ctx, fixture.projectID, mrIID))
	if checks.Mergeable || checks.Status != projectpipelinerepo.StatusFailed || checks.Pipeline == nil {
		t.Fatalf("expected failed checks: %+v", checks)
	}
	testutil.RequireNoError(t, fixture.pipelineRepository.UpdateStatus(fixture.ctx, pipeline, projectpipelinerepo.StatusSucceeded), "mark pipeline succeeded")
}

func assertMergeAfterSuccessfulPipeline(t *testing.T, fixture mergeRequestFixture, mrIID int64) {
	t.Helper()
	assertMergeRequestMerge(t, fixture, mrIID)
}

func assertTargetBranchPipeline(t *testing.T, fixture mergeRequestFixture) {
	t.Helper()

	targetBranch := findBranch(fixture.ctx, t, fixture.gitRepository, fixture.projectFullPath+".git", fixture.projectDefaultBranch, "main")
	targetPipeline := testutil.Must(fixture.pipelineRepository.GetLatestByProjectRefCommit(fixture.ctx, fixture.projectID, "main", targetBranch.Hash))
	if targetPipeline.Source != "push" || targetPipeline.CommitSHA != targetBranch.Hash {
		t.Fatalf("unexpected target branch pipeline: %+v", targetPipeline)
	}
}
