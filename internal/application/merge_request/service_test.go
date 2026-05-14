package mergerequest_test

import (
	"context"
	"fmt"
	"log/slog"
	"path/filepath"
	"strings"
	"testing"

	jobservice "github.com/lyonbrown4d/gity/internal/application/job"
	mergerequestservice "github.com/lyonbrown4d/gity/internal/application/merge_request"
	organizationservice "github.com/lyonbrown4d/gity/internal/application/organization"
	pipelineservice "github.com/lyonbrown4d/gity/internal/application/pipeline"
	projectservice "github.com/lyonbrown4d/gity/internal/application/project"
	userservice "github.com/lyonbrown4d/gity/internal/application/user"
	"github.com/lyonbrown4d/gity/internal/config"
	"github.com/lyonbrown4d/gity/internal/infrastructure/git_exec"
	"github.com/lyonbrown4d/gity/internal/infrastructure/git_repo"
	"github.com/lyonbrown4d/gity/internal/infrastructure/persistence/core"
	organizationrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/organization"
	organizationmemberrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/organization_member"
	projectrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project"
	projectbranchprotectionrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_branch_protection"
	projectjobrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_job"
	projectmergerequestrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_merge_request"
	projectmergerequestapprovalrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_merge_request_approval"
	projectmergerequestapprovalrulerepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_merge_request_approval_rule"
	projectmergerequestcommentrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_merge_request_comment"
	projectmergerequestparticipantrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_merge_request_participant"
	projectpipelinerepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_pipeline"
	projectpipelinejobrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_pipeline_job"
	userrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/user"
	usertokenrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/user_token"
	"github.com/lyonbrown4d/gity/internal/testutil"

	"github.com/arcgolabs/dbx"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestMergeRequestFlow(t *testing.T) {
	t.Parallel()

	fixture := newMergeRequestFixture(t, false)
	mrIID := assertCreateMergeRequest(t, fixture, "merge feature")
	assertMergeRequestDiff(t, fixture, mrIID)
	assertMergeRequestCollaboration(t, fixture, mrIID)
	assertMergeRequestMerge(t, fixture, mrIID)
	assertCloseSecondMergeRequest(t, fixture)
	assertListMergeRequests(t, fixture)
}

type mergeRequestFixture struct {
	ctx                    context.Context
	repoRoot               string
	projectID              int64
	projectFullPath        string
	projectDefaultBranch   string
	ownerID                int64
	reviewerID             int64
	runner                 *gitexec.Runner
	gitRepository          *gitrepo.Service
	projectService         *projectservice.Service
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
	mergeRequestParticipantRepository := testutil.Must(projectmergerequestparticipantrepo.NewRepository(db))
	mergeRequestCommentRepository := testutil.Must(projectmergerequestcommentrepo.NewRepository(db))
	mergeRequestApprovalRepository := testutil.Must(projectmergerequestapprovalrepo.NewRepository(db))
	mergeRequestApprovalRuleRepository := testutil.Must(projectmergerequestapprovalrulerepo.NewRepository(db))
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
	mergeRequestSvc := mergerequestservice.NewServiceWithDependencies(
		logger,
		mergerequestservice.NewRepositories(projectRepository, mergeRequestRepository, userRepository, organizationMemberRepository, nil, projectBranchProtectionRepository),
		mergerequestservice.NewCollaborationRepositories(mergeRequestParticipantRepository, mergeRequestCommentRepository, mergeRequestApprovalRepository, mergeRequestApprovalRuleRepository),
		mergerequestservice.NewGitDependencies(gitRepository, runner),
		mergerequestservice.NewPipelineDeps(pipelineRepository, pipelineSvc),
		nil,
	)

	owner := testutil.Must(userSvc.Create(ctx, userservice.CreateInput{Username: "alice", DisplayName: "Alice", Email: "alice@gity.dev"}))
	reviewer := testutil.Must(userSvc.Create(ctx, userservice.CreateInput{Username: "bob", DisplayName: "Bob", Email: "bob@gity.dev"}))
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
		reviewerID:             reviewer.ID,
		runner:                 runner,
		gitRepository:          gitRepository,
		projectService:         projectSvc,
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
	return pipelineservice.NewService(projectRepository, pipelineRepository, pipelineJobRepository, jobSvc, jobRepository, gitRepository, nil)
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

func assertMergeRequestCollaboration(t *testing.T, fixture mergeRequestFixture, mrIID int64) {
	t.Helper()

	commented := testutil.Must(fixture.mergeRequestService.CreateComment(fixture.ctx, fixture.projectID, mrIID, mergerequestservice.CommentInput{AuthorUserID: fixture.reviewerID, Body: "Looks good"}))
	if len(commented.Comments) != 1 || commented.Comments[0].Body != "Looks good" {
		t.Fatalf("unexpected merge request comments: %+v", commented.Comments)
	}
	comments := testutil.Must(fixture.mergeRequestService.ListComments(fixture.ctx, fixture.projectID, mrIID))
	if len(comments.Comments) != 1 || comments.Comments[0].AuthorUserID != fixture.reviewerID {
		t.Fatalf("unexpected listed merge request comments: %+v", comments.Comments)
	}
	approved := testutil.Must(fixture.mergeRequestService.Approve(fixture.ctx, fixture.projectID, mrIID, mergerequestservice.ApprovalInput{UserID: fixture.reviewerID}))
	if len(approved.Approvals) != 1 || approved.Approvals[0].UserID != fixture.reviewerID {
		t.Fatalf("unexpected merge request approvals: %+v", approved.Approvals)
	}
	unapproved := testutil.Must(fixture.mergeRequestService.Unapprove(fixture.ctx, fixture.projectID, mrIID, mergerequestservice.ApprovalInput{UserID: fixture.reviewerID}))
	if len(unapproved.Approvals) != 0 {
		t.Fatalf("unexpected merge request approvals after unapprove: %+v", unapproved.Approvals)
	}
}

func assertMergeRequestMerge(t *testing.T, fixture mergeRequestFixture, mrIID int64) {
	t.Helper()

	merged := testutil.Must(fixture.mergeRequestService.Merge(fixture.ctx, fixture.projectID, mrIID, mergerequestservice.MergeInput{AuthorName: "Gity Test", AuthorEmail: "test@gity.dev", ActorUserID: fixture.ownerID}))
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
