package job_test

import (
	"context"
	"encoding/base64"
	"fmt"
	"log/slog"
	"path/filepath"
	"testing"
	"time"

	"github.com/DaiYuANg/gity/internal/config"
	dbx "github.com/DaiYuANg/gity/internal/dbxcompat"
	platformstorage "github.com/DaiYuANg/gity/internal/platform/storage"
	"github.com/DaiYuANg/gity/internal/repository/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/repository/namespace"
	projectrepo "github.com/DaiYuANg/gity/internal/repository/project"
	projectjobrepo "github.com/DaiYuANg/gity/internal/repository/projectjob"
	projectjobartifactrepo "github.com/DaiYuANg/gity/internal/repository/projectjobartifact"
	projectjoblogrepo "github.com/DaiYuANg/gity/internal/repository/projectjoblog"
	jobservice "github.com/DaiYuANg/gity/internal/service/job"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestProjectJobFlow(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	db := openTestDB(t)
	defer db.Close()
	if err := core.EnsureSchema(ctx, db); err != nil {
		t.Fatalf("ensure schema: %v", err)
	}

	namespaceRepository, err := namespacerepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new namespace repo: %v", err)
	}
	projectRepository, err := projectrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new project repo: %v", err)
	}
	jobRepository, err := projectjobrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new job repo: %v", err)
	}
	service := jobservice.NewService(slog.Default(), projectRepository, jobRepository, nil, nil, nil)

	namespace, err := namespaceRepository.Create(ctx, namespacerepo.CreateInput{Kind: "group", Name: "Core Team", PathKey: "core-team"})
	if err != nil {
		t.Fatalf("create namespace: %v", err)
	}
	project, err := projectRepository.Create(ctx, projectrepo.CreateInput{NamespaceID: namespace.ID, Name: "Gity", PathKey: "gity", Visibility: "private"}, namespace)
	if err != nil {
		t.Fatalf("create project: %v", err)
	}

	created, err := service.EnqueueProjectJob(ctx, project.ID, jobservice.CreateInput{Payload: `{"reason":"test"}`})
	if err != nil {
		t.Fatalf("enqueue job: %v", err)
	}
	if created.Status != projectjobrepo.StatusPending || created.Kind != jobservice.KindNoop || created.MaxAttempts != 3 {
		t.Fatalf("unexpected created job: %+v", created)
	}

	items, err := service.ListProjectJobs(ctx, project.ID)
	if err != nil {
		t.Fatalf("list jobs: %v", err)
	}
	if len(items) != 1 || items[0].ID != created.ID {
		t.Fatalf("unexpected jobs: %+v", items)
	}

	claimed, err := service.RunNext(ctx, "worker-a", time.Minute)
	if err != nil {
		t.Fatalf("run next: %v", err)
	}
	if !claimed {
		t.Fatalf("expected a job to be claimed")
	}
	completed, err := service.GetProjectJob(ctx, project.ID, created.ID)
	if err != nil {
		t.Fatalf("get completed job: %v", err)
	}
	if completed.Status != projectjobrepo.StatusSucceeded || completed.Attempts != 1 || completed.Result == "" {
		t.Fatalf("unexpected completed job: %+v", completed)
	}

	claimed, err = service.RunNext(ctx, "worker-a", time.Minute)
	if err != nil {
		t.Fatalf("run next without work: %v", err)
	}
	if claimed {
		t.Fatalf("expected no job to be claimed")
	}
}

func TestProjectScriptJobTraceAndArtifacts(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	db := openTestDB(t)
	defer db.Close()
	if err := core.EnsureSchema(ctx, db); err != nil {
		t.Fatalf("ensure schema: %v", err)
	}

	namespaceRepository, err := namespacerepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new namespace repo: %v", err)
	}
	projectRepository, err := projectrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new project repo: %v", err)
	}
	jobRepository, err := projectjobrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new job repo: %v", err)
	}
	logRepository, err := projectjoblogrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new job log repo: %v", err)
	}
	artifactRepository, err := projectjobartifactrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new artifact repo: %v", err)
	}
	storageSvc, err := platformstorage.NewService(config.Settings{Storage: config.StorageSettings{Driver: "local", Root: filepath.Join(t.TempDir(), "storage")}})
	if err != nil {
		t.Fatalf("new storage service: %v", err)
	}
	service := jobservice.NewService(slog.Default(), projectRepository, jobRepository, logRepository, artifactRepository, storageSvc)

	namespace, err := namespaceRepository.Create(ctx, namespacerepo.CreateInput{Kind: "group", Name: "Core Team", PathKey: "core-team"})
	if err != nil {
		t.Fatalf("create namespace: %v", err)
	}
	project, err := projectRepository.Create(ctx, projectrepo.CreateInput{NamespaceID: namespace.ID, Name: "Gity", PathKey: "gity", Visibility: "private"}, namespace)
	if err != nil {
		t.Fatalf("create project: %v", err)
	}
	created, err := service.EnqueueProjectJob(ctx, project.ID, jobservice.CreateInput{
		Kind:    jobservice.KindScript,
		Payload: `{"script":["go test ./..."]}`,
	})
	if err != nil {
		t.Fatalf("enqueue script job: %v", err)
	}
	claimed, ok, err := service.ClaimProjectJob(ctx, project.ID, "runner-a", time.Minute)
	if err != nil {
		t.Fatalf("claim script job: %v", err)
	}
	if !ok || claimed.ID != created.ID {
		t.Fatalf("unexpected claimed job: ok=%v job=%+v", ok, claimed)
	}
	if _, err := service.CompleteProjectJob(ctx, project.ID, claimed.ID, `{"exit_code":0,"output":"ok\n","output_truncated":false,"duration_millis":42,"work_dir":"."}`); err != nil {
		t.Fatalf("complete script job: %v", err)
	}
	trace, err := service.GetProjectJobTrace(ctx, project.ID, claimed.ID)
	if err != nil {
		t.Fatalf("get trace: %v", err)
	}
	if trace.Trace != "ok" || trace.ExitCode != 0 || trace.DurationMillis != 42 {
		t.Fatalf("unexpected trace: %+v", trace)
	}
	artifact, err := service.UploadProjectJobArtifact(ctx, project.ID, claimed.ID, jobservice.UploadArtifactInput{
		FileName:      "report.txt",
		FilePath:      "dist/report.txt",
		ContentBase64: base64.StdEncoding.EncodeToString([]byte("artifact")),
	})
	if err != nil {
		t.Fatalf("upload artifact: %v", err)
	}
	items, err := service.ListProjectJobArtifacts(ctx, project.ID, claimed.ID)
	if err != nil {
		t.Fatalf("list artifacts: %v", err)
	}
	if len(items) != 1 || items[0].ID != artifact.ID {
		t.Fatalf("unexpected artifacts: %+v", items)
	}
	content, err := service.GetProjectJobArtifactContent(ctx, project.ID, claimed.ID, artifact.ID)
	if err != nil {
		t.Fatalf("get artifact content: %v", err)
	}
	decoded, err := base64.StdEncoding.DecodeString(content.ContentBase64)
	if err != nil {
		t.Fatalf("decode artifact content: %v", err)
	}
	if string(decoded) != "artifact" {
		t.Fatalf("artifact content = %q", decoded)
	}
}

func TestProjectJobRetry(t *testing.T) {
	t.Parallel()

	ctx := context.Background()
	db := openTestDB(t)
	defer db.Close()
	if err := core.EnsureSchema(ctx, db); err != nil {
		t.Fatalf("ensure schema: %v", err)
	}

	namespaceRepository, err := namespacerepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new namespace repo: %v", err)
	}
	projectRepository, err := projectrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new project repo: %v", err)
	}
	jobRepository, err := projectjobrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new job repo: %v", err)
	}
	service := jobservice.NewService(slog.Default(), projectRepository, jobRepository, nil, nil, nil)

	namespace, err := namespaceRepository.Create(ctx, namespacerepo.CreateInput{Kind: "group", Name: "Core Team", PathKey: "core-team"})
	if err != nil {
		t.Fatalf("create namespace: %v", err)
	}
	project, err := projectRepository.Create(ctx, projectrepo.CreateInput{NamespaceID: namespace.ID, Name: "Gity", PathKey: "gity", Visibility: "private"}, namespace)
	if err != nil {
		t.Fatalf("create project: %v", err)
	}

	created, err := service.EnqueueProjectJob(ctx, project.ID, jobservice.CreateInput{
		Kind:    jobservice.KindScript,
		Payload: `{"script":["echo"]}`,
		MaxAttempts: 2,
	})
	if err != nil {
		t.Fatalf("enqueue job: %v", err)
	}
	claimed, ok, err := service.ClaimProjectJob(ctx, project.ID, "runner-a", time.Minute)
	if err != nil {
		t.Fatalf("claim job: %v", err)
	}
	if !ok || claimed.ID != created.ID {
		t.Fatalf("unexpected claim result: ok=%v job=%+v", ok, claimed)
	}
	if _, err := service.FailProjectJob(ctx, project.ID, claimed.ID, "failed", time.Second); err != nil {
		t.Fatalf("fail job: %v", err)
	}
	retried, err := service.RetryProjectJob(ctx, project.ID, claimed.ID)
	if err != nil {
		t.Fatalf("retry job: %v", err)
	}
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
