package job_test

import (
	"context"
	"encoding/base64"
	"fmt"
	"log/slog"
	"path/filepath"
	"testing"
	"time"

	jobservice "github.com/lyonbrown4d/gity/internal/application/job"
	"github.com/lyonbrown4d/gity/internal/config"
	"github.com/lyonbrown4d/gity/internal/infrastructure/persistence/core"
	organizationrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/organization"
	projectrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project"
	projectjobrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_job"
	projectjobartifactrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_job_artifact"
	projectjoblogrepo "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/project_job_log"
	infrastorage "github.com/lyonbrown4d/gity/internal/infrastructure/storage"
	"github.com/lyonbrown4d/gity/internal/testutil"

	"github.com/arcgolabs/dbx"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestProjectJobFlow(t *testing.T) {
	t.Parallel()

	fixture := newJobFixture(t, false)
	createdID := assertEnqueueNoopJob(t, fixture)
	assertListProjectJobs(t, fixture, createdID)
	assertRunNextCompletesNoopJob(t, fixture, createdID)
	assertRunNextWithoutWork(t, fixture)
}

type jobFixture struct {
	ctx           context.Context
	projectID     int64
	jobRepository *projectjobrepo.Repository
	service       *jobservice.Service
}

func newJobFixture(t *testing.T, withArtifacts bool) jobFixture {
	t.Helper()

	ctx := context.Background()
	db := openTestDB(t)
	testutil.CleanupClose(t, "db", db)
	testutil.RequireNoError(t, core.EnsureSchema(ctx, db), "ensure schema")

	organizationRepository := testutil.Must(organizationrepo.NewRepository(db))
	projectRepository := testutil.Must(projectrepo.NewRepository(db))
	jobRepository := testutil.Must(projectjobrepo.NewRepository(db))
	logRepository := createJobLogRepository(t, db, withArtifacts)
	artifactRepository := createJobArtifactRepository(t, db, withArtifacts)
	storageSvc := createJobStorage(t, withArtifacts)
	service := jobservice.NewService(slog.Default(), projectRepository, jobRepository, logRepository, artifactRepository, storageSvc)

	organization := testutil.Must(organizationRepository.Create(ctx, organizationrepo.CreateInput{Name: "Core Team", PathKey: "core-team"}))
	project := testutil.Must(projectRepository.Create(ctx, projectrepo.CreateInput{OrganizationID: organization.ID, Name: "Gity", PathKey: "gity", Visibility: "private"}, organization))

	return jobFixture{ctx: ctx, projectID: project.ID, jobRepository: jobRepository, service: service}
}

func createJobLogRepository(t *testing.T, db *dbx.DB, enabled bool) *projectjoblogrepo.Repository {
	t.Helper()
	if !enabled {
		return nil
	}
	return testutil.Must(projectjoblogrepo.NewRepository(db))
}

func createJobArtifactRepository(t *testing.T, db *dbx.DB, enabled bool) *projectjobartifactrepo.Repository {
	t.Helper()
	if !enabled {
		return nil
	}
	return testutil.Must(projectjobartifactrepo.NewRepository(db))
}

func createJobStorage(t *testing.T, enabled bool) *infrastorage.Service {
	t.Helper()
	if !enabled {
		return nil
	}
	return testutil.Must(infrastorage.NewService(config.Settings{Storage: config.StorageSettings{Driver: "local", Root: filepath.Join(t.TempDir(), "storage")}}))
}

func assertEnqueueNoopJob(t *testing.T, fixture jobFixture) int64 {
	t.Helper()

	created := testutil.Must(fixture.service.EnqueueProjectJob(fixture.ctx, fixture.projectID, jobservice.CreateInput{Payload: `{"reason":"test"}`}))
	if created.Status != projectjobrepo.StatusPending || created.Kind != jobservice.KindNoop || created.MaxAttempts != 3 {
		t.Fatalf("unexpected created job: %+v", created)
	}
	return created.ID
}

func assertListProjectJobs(t *testing.T, fixture jobFixture, jobID int64) {
	t.Helper()

	items := testutil.Must(fixture.service.ListProjectJobs(fixture.ctx, fixture.projectID))
	if len(items) != 1 || items[0].ID != jobID {
		t.Fatalf("unexpected jobs: %+v", items)
	}
}

func assertRunNextCompletesNoopJob(t *testing.T, fixture jobFixture, jobID int64) {
	t.Helper()

	claimed, err := fixture.service.RunNext(fixture.ctx, "worker-a", time.Minute)
	testutil.RequireNoError(t, err, "run next")
	if !claimed {
		t.Fatalf("expected a job to be claimed")
	}
	completed := testutil.Must(fixture.service.GetProjectJob(fixture.ctx, fixture.projectID, jobID))
	if completed.Status != projectjobrepo.StatusSucceeded || completed.Attempts != 1 || completed.Result == "" {
		t.Fatalf("unexpected completed job: %+v", completed)
	}
}

func assertRunNextWithoutWork(t *testing.T, fixture jobFixture) {
	t.Helper()

	claimed, err := fixture.service.RunNext(fixture.ctx, "worker-a", time.Minute)
	testutil.RequireNoError(t, err, "run next without work")
	if claimed {
		t.Fatalf("expected no job to be claimed")
	}
}

func TestProjectScriptJobTraceAndArtifacts(t *testing.T) {
	t.Parallel()

	fixture := newJobFixture(t, true)
	claimedID := assertClaimScriptJob(t, fixture)
	assertAppendProjectJobTrace(t, fixture, claimedID)
	assertCompleteScriptJob(t, fixture, claimedID)
	artifactID := assertUploadProjectJobArtifact(t, fixture, claimedID)
	assertListProjectJobArtifacts(t, fixture, claimedID, artifactID)
	assertProjectJobArtifactContent(t, fixture, claimedID, artifactID)
}

func assertClaimScriptJob(t *testing.T, fixture jobFixture) int64 {
	t.Helper()

	created := testutil.Must(fixture.service.EnqueueProjectJob(fixture.ctx, fixture.projectID, jobservice.CreateInput{
		Kind:    jobservice.KindScript,
		Payload: `{"script":["go test ./..."]}`,
	}))
	claimed, ok, err := fixture.service.ClaimProjectJob(fixture.ctx, fixture.projectID, "runner-a", time.Minute)
	testutil.RequireNoError(t, err, "claim script job")
	if !ok || claimed.ID != created.ID {
		t.Fatalf("unexpected claimed job: ok=%v job=%+v", ok, claimed)
	}
	return claimed.ID
}

func assertAppendProjectJobTrace(t *testing.T, fixture jobFixture, jobID int64) {
	t.Helper()

	streamed := testutil.Must(fixture.service.AppendProjectJobTrace(fixture.ctx, fixture.projectID, jobID, jobservice.AppendTraceInput{Output: "streaming\n", DurationMillis: 12}))
	if streamed.Trace != "streaming" || streamed.DurationMillis != 12 {
		t.Fatalf("unexpected streamed trace: %+v", streamed)
	}
	page := testutil.Must(fixture.service.GetProjectJobTracePage(fixture.ctx, fixture.projectID, jobID, jobservice.TracePageInput{Offset: 3, Limit: 4}))
	if page.Trace != "eami" || page.TraceTotal != len("streaming") || !page.HasMore {
		t.Fatalf("unexpected paged trace: %+v", page)
	}
}

func assertCompleteScriptJob(t *testing.T, fixture jobFixture, jobID int64) {
	t.Helper()

	_, completeErr := fixture.service.CompleteProjectJob(fixture.ctx, fixture.projectID, jobID, `{"exit_code":0,"output":"ok\n","output_truncated":false,"duration_millis":42,"work_dir":"."}`)
	testutil.RequireNoError(t, completeErr, "complete script job")
	trace := testutil.Must(fixture.service.GetProjectJobTrace(fixture.ctx, fixture.projectID, jobID))
	if trace.Trace != "ok" || trace.ExitCode != 0 || trace.DurationMillis != 42 {
		t.Fatalf("unexpected trace: %+v", trace)
	}
}

func assertUploadProjectJobArtifact(t *testing.T, fixture jobFixture, jobID int64) int64 {
	t.Helper()

	artifact := testutil.Must(fixture.service.UploadProjectJobArtifact(fixture.ctx, fixture.projectID, jobID, jobservice.UploadArtifactInput{
		FileName:      "report.txt",
		FilePath:      "dist/report.txt",
		ContentBase64: base64.StdEncoding.EncodeToString([]byte("artifact")),
	}))
	return artifact.ID
}

func assertListProjectJobArtifacts(t *testing.T, fixture jobFixture, jobID, artifactID int64) {
	t.Helper()

	items := testutil.Must(fixture.service.ListProjectJobArtifacts(fixture.ctx, fixture.projectID, jobID))
	if len(items) != 1 || items[0].ID != artifactID {
		t.Fatalf("unexpected artifacts: %+v", items)
	}
}

func assertProjectJobArtifactContent(t *testing.T, fixture jobFixture, jobID, artifactID int64) {
	t.Helper()

	content := testutil.Must(fixture.service.GetProjectJobArtifactContent(fixture.ctx, fixture.projectID, jobID, artifactID))
	decoded, err := base64.StdEncoding.DecodeString(content.ContentBase64)
	testutil.RequireNoError(t, err, "decode artifact content")
	if string(decoded) != "artifact" {
		t.Fatalf("artifact content = %q", decoded)
	}
}

func TestProjectJobRetry(t *testing.T) {
	t.Parallel()

	fixture := newJobFixture(t, true)
	claimedID := assertClaimRetryableJob(t, fixture)
	assertRetryProjectJob(t, fixture, claimedID)
}

func TestProjectJobExpiredLeaseIsRequeued(t *testing.T) {
	t.Parallel()

	fixture := newJobFixture(t, true)
	created := testutil.Must(fixture.service.EnqueueProjectJob(fixture.ctx, fixture.projectID, jobservice.CreateInput{
		Kind:        jobservice.KindScript,
		Payload:     `{"script":["echo lease"]}`,
		MaxAttempts: 2,
	}))
	claimed, ok, err := fixture.service.ClaimProjectJob(fixture.ctx, fixture.projectID, "runner-a", time.Minute)
	testutil.RequireNoError(t, err, "claim job")
	if !ok || claimed.ID != created.ID {
		t.Fatalf("unexpected claim result: ok=%v job=%+v", ok, claimed)
	}
	expired := testutil.Must(fixture.jobRepository.RequeueExpiredLeases(fixture.ctx, time.Now().UTC().Add(time.Hour)))
	if expired != 1 {
		t.Fatalf("expected one expired lease, got %d", expired)
	}
	requeued := testutil.Must(fixture.service.GetProjectJob(fixture.ctx, fixture.projectID, created.ID))
	if requeued.Status != projectjobrepo.StatusPending || requeued.LastError != "runner lease expired" {
		t.Fatalf("unexpected requeued job: %+v", requeued)
	}
}

func assertClaimRetryableJob(t *testing.T, fixture jobFixture) int64 {
	t.Helper()

	created := testutil.Must(fixture.service.EnqueueProjectJob(fixture.ctx, fixture.projectID, jobservice.CreateInput{
		Kind:        jobservice.KindScript,
		Payload:     `{"script":["echo"]}`,
		MaxAttempts: 2,
	}))
	claimed, ok, err := fixture.service.ClaimProjectJob(fixture.ctx, fixture.projectID, "runner-a", time.Minute)
	testutil.RequireNoError(t, err, "claim job")
	if !ok || claimed.ID != created.ID {
		t.Fatalf("unexpected claim result: ok=%v job=%+v", ok, claimed)
	}
	_, failErr := fixture.service.FailProjectJob(fixture.ctx, fixture.projectID, claimed.ID, "failed", time.Second)
	testutil.RequireNoError(t, failErr, "fail job")
	return claimed.ID
}

func assertRetryProjectJob(t *testing.T, fixture jobFixture, jobID int64) {
	t.Helper()

	retried := testutil.Must(fixture.service.RetryProjectJob(fixture.ctx, fixture.projectID, jobID))
	if retried.Status != projectjobrepo.StatusPending {
		t.Fatalf("unexpected job status: %s", retried.Status)
	}
	if retried.Attempts != 0 {
		t.Fatalf("job attempts should reset on retry, got %d", retried.Attempts)
	}
	if retried.Result != "" || retried.LastError != "" {
		t.Fatalf("retry should clear result and error: %+v", retried)
	}
}

func openTestDB(t *testing.T) *dbx.DB {
	t.Helper()
	dbPath := filepath.Join(t.TempDir(), "gity-test.db")
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
