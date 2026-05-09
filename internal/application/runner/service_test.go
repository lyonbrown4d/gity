package runner_test

import (
	"archive/zip"
	"bytes"
	"context"
	"encoding/base64"
	"fmt"
	"log/slog"
	"path/filepath"
	"strings"
	"testing"
	"time"

	jobservice "github.com/DaiYuANg/gity/internal/application/job"
	runnerservice "github.com/DaiYuANg/gity/internal/application/runner"
	"github.com/DaiYuANg/gity/internal/config"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_exec"
	"github.com/DaiYuANg/gity/internal/infrastructure/persistence/core"
	organizationrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/organization"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectjobrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_job"
	projectjoblogrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_job_log"
	projectrunnerrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_runner"
	"github.com/DaiYuANg/gity/internal/testutil"

	"github.com/arcgolabs/dbx"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestProjectRunnerFlow(t *testing.T) {
	t.Parallel()

	fixture := newRunnerFixture(t)
	token, runnerID := assertRegisterProjectRunner(t, fixture)
	fixture.runnerToken = token
	assertListProjectRunners(t, fixture, runnerID)
	assertRunnerHeartbeat(t, fixture)
	assertInternalNoopJob(t, fixture)
	assertScriptJobLifecycle(t, fixture)
	assertRetryableJobFailure(t, fixture)
}

type runnerFixture struct {
	ctx             context.Context
	projectID       int64
	projectFullPath string
	runnerToken     string
	jobService      *jobservice.Service
	runnerService   *runnerservice.Service
}

func newRunnerFixture(t *testing.T) runnerFixture {
	t.Helper()

	ctx := context.Background()
	db := openTestDB(t)
	testutil.CleanupClose(t, "db", db)
	testutil.RequireNoError(t, core.EnsureSchema(ctx, db), "ensure schema")

	organizationRepository := testutil.Must(organizationrepo.NewRepository(db))
	projectRepository := testutil.Must(projectrepo.NewRepository(db))
	jobRepository := testutil.Must(projectjobrepo.NewRepository(db))
	logRepository := testutil.Must(projectjoblogrepo.NewRepository(db))
	runnerRepository := testutil.Must(projectrunnerrepo.NewRepository(db))
	repoRoot := filepath.Join(t.TempDir(), "repos")
	gitRunner := gitexec.NewRunner(config.Settings{Git: config.GitSettings{Bin: "git", RepoRoot: repoRoot}})
	jobSvc := jobservice.NewService(slog.Default(), projectRepository, jobRepository, logRepository, nil, nil)
	runnerSvc := runnerservice.NewService(projectRepository, runnerRepository, jobSvc, nil, gitRunner)

	organization := testutil.Must(organizationRepository.Create(ctx, organizationrepo.CreateInput{Name: "Core Team", PathKey: "core-team"}))
	project := testutil.Must(projectRepository.Create(ctx, projectrepo.CreateInput{OrganizationID: organization.ID, Name: "Gity", PathKey: "gity", Visibility: "private"}, organization))
	testutil.RequireNoError(t, gitRunner.InitBare(ctx, project.FullPath+".git", "main"), "init bare repo")
	testutil.RequireNoError(t, gitRunner.CreateFileCommit(ctx, project.FullPath+".git", gitexec.CreateFileCommitInput{
		BranchName:  "main",
		FilePath:    "README.md",
		Content:     "hello source archive\n",
		Message:     "Add README",
		AuthorName:  "Gity Test",
		AuthorEmail: "test@gity.dev",
	}), "create source commit")

	return runnerFixture{
		ctx:             ctx,
		projectID:       project.ID,
		projectFullPath: project.FullPath,
		jobService:      jobSvc,
		runnerService:   runnerSvc,
	}
}

func assertRegisterProjectRunner(t *testing.T, fixture runnerFixture) (string, int64) {
	t.Helper()

	registration := testutil.Must(fixture.runnerService.RegisterProjectRunner(fixture.ctx, fixture.projectID, runnerservice.RegisterInput{Name: "linux-amd64", Tags: "go, linux, go"}))
	if !strings.HasPrefix(registration.Token, "grt_") || registration.Runner.Name != "linux-amd64" || registration.Runner.Tags != "go,linux" {
		t.Fatalf("unexpected registration: %+v", registration)
	}
	return registration.Token, registration.Runner.ID
}

func assertListProjectRunners(t *testing.T, fixture runnerFixture, runnerID int64) {
	t.Helper()

	runners := testutil.Must(fixture.runnerService.ListProjectRunners(fixture.ctx, fixture.projectID))
	if len(runners) != 1 || runners[0].ID != runnerID || runners[0].Active != true {
		t.Fatalf("unexpected runners: %+v", runners)
	}
}

func assertRunnerHeartbeat(t *testing.T, fixture runnerFixture) {
	t.Helper()

	heartbeat := testutil.Must(fixture.runnerService.Heartbeat(fixture.ctx, fixture.runnerToken))
	if heartbeat.Status != projectrunnerrepo.StatusOnline || heartbeat.LastContactAt == nil {
		t.Fatalf("unexpected heartbeat view: %+v", heartbeat)
	}
}

func assertInternalNoopJob(t *testing.T, fixture runnerFixture) {
	t.Helper()

	created := testutil.Must(fixture.jobService.EnqueueProjectJob(fixture.ctx, fixture.projectID, jobservice.CreateInput{Payload: `{"runner":"test"}`}))
	noopClaim := testutil.Must(fixture.runnerService.ClaimJob(fixture.ctx, fixture.runnerToken, time.Minute))
	if noopClaim.Claimed {
		t.Fatalf("runner should not claim internal noop jobs: %+v", noopClaim)
	}
	_, runErr := fixture.jobService.RunNext(fixture.ctx, "worker-a", time.Minute)
	testutil.RequireNoError(t, runErr, "run internal noop job")
	completedNoop := testutil.Must(fixture.jobService.GetProjectJob(fixture.ctx, fixture.projectID, created.ID))
	if completedNoop.Status != projectjobrepo.StatusSucceeded {
		t.Fatalf("unexpected internal noop job status: %+v", completedNoop)
	}
}

func assertScriptJobLifecycle(t *testing.T, fixture runnerFixture) {
	t.Helper()

	created := testutil.Must(fixture.jobService.EnqueueProjectJob(fixture.ctx, fixture.projectID, jobservice.CreateInput{
		Kind:    jobservice.KindScript,
		Payload: fmt.Sprintf(`{"project_full_path":%q,"ref_name":"main","script":["echo hello"]}`, fixture.projectFullPath),
	}))
	claim := testutil.Must(fixture.runnerService.ClaimJob(fixture.ctx, fixture.runnerToken, time.Minute))
	if !claim.Claimed || claim.Job.ID != created.ID || claim.Job.Status != projectjobrepo.StatusRunning {
		t.Fatalf("unexpected claim: %+v", claim)
	}
	trace := testutil.Must(fixture.runnerService.AppendTrace(fixture.ctx, fixture.runnerToken, created.ID, runnerservice.AppendTraceInput{Output: "hello\n", DurationMillis: 10}))
	if trace.Trace != "hello" || trace.DurationMillis != 10 {
		t.Fatalf("unexpected runner trace: %+v", trace)
	}
	assertSourceArchive(t, fixture, created.ID)

	completed := testutil.Must(fixture.runnerService.CompleteJob(fixture.ctx, fixture.runnerToken, created.ID, `{"ok":true}`))
	if completed.Status != projectjobrepo.StatusSucceeded || completed.Result == "" {
		t.Fatalf("unexpected completed job: %+v", completed)
	}
}

func assertSourceArchive(t *testing.T, fixture runnerFixture, jobID int64) {
	t.Helper()

	archive := testutil.Must(fixture.runnerService.DownloadSourceArchive(fixture.ctx, fixture.runnerToken, jobID))
	if archive.Encoding != "base64" || archive.FileName == "" {
		t.Fatalf("unexpected source archive metadata: %+v", archive)
	}
	content, err := base64.StdEncoding.DecodeString(archive.ContentBase64)
	testutil.RequireNoError(t, err, "decode source archive")
	if !zipContainsFile(t, content, "README.md", "hello source archive") {
		t.Fatalf("source archive does not contain expected README")
	}
}

func assertRetryableJobFailure(t *testing.T, fixture runnerFixture) {
	t.Helper()

	retryable := testutil.Must(fixture.jobService.EnqueueProjectJob(fixture.ctx, fixture.projectID, jobservice.CreateInput{Kind: jobservice.KindScript, Payload: `{"script":["echo retry"]}`, MaxAttempts: 2}))
	_, claimErr := fixture.runnerService.ClaimJob(fixture.ctx, fixture.runnerToken, time.Minute)
	testutil.RequireNoError(t, claimErr, "claim retryable job")
	failed := testutil.Must(fixture.runnerService.FailJob(fixture.ctx, fixture.runnerToken, retryable.ID, "executor failed", "", time.Millisecond))
	if failed.Status != projectjobrepo.StatusPending || failed.LastError != "executor failed" {
		t.Fatalf("unexpected retryable failure state: %+v", failed)
	}
}

func zipContainsFile(t *testing.T, content []byte, fileName, contains string) bool {
	t.Helper()
	reader, err := zip.NewReader(bytes.NewReader(content), int64(len(content)))
	if err != nil {
		t.Fatalf("open source archive: %v", err)
	}
	for _, file := range reader.File {
		if file.Name == fileName {
			return zipFileContains(t, file, contains)
		}
	}
	return false
}

func zipFileContains(t *testing.T, file *zip.File, contains string) bool {
	t.Helper()
	rc := testutil.Must(file.Open())
	defer func() {
		if err := rc.Close(); err != nil {
			t.Fatalf("close source archive file: %v", err)
		}
	}()
	var buffer bytes.Buffer
	_, err := buffer.ReadFrom(rc)
	testutil.RequireNoError(t, err, "read source archive file")
	return strings.Contains(buffer.String(), contains)
}

func openTestDB(t *testing.T) *dbx.DB {
	t.Helper()
	dbPath := filepath.Join(t.TempDir(), "gity-runner-test.db")
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
