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
	"github.com/DaiYuANg/gity/internal/infrastructure/gitexec"
	"github.com/DaiYuANg/gity/internal/infrastructure/persistence/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespace"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectjobrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectjob"
	projectjoblogrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectjoblog"
	projectrunnerrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/projectrunner"

	"github.com/arcgolabs/dbx"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func TestProjectRunnerFlow(t *testing.T) {
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
	runnerRepository, err := projectrunnerrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new runner repo: %v", err)
	}
	repoRoot := filepath.Join(t.TempDir(), "repos")
	gitRunner := gitexec.NewRunner(config.Settings{Git: config.GitSettings{Bin: "git", RepoRoot: repoRoot}})
	jobSvc := jobservice.NewService(slog.Default(), projectRepository, jobRepository, logRepository, nil, nil)
	runnerSvc := runnerservice.NewService(projectRepository, runnerRepository, jobSvc, nil, gitRunner)

	namespace, err := namespaceRepository.Create(ctx, namespacerepo.CreateInput{Kind: "group", Name: "Core Team", PathKey: "core-team"})
	if err != nil {
		t.Fatalf("create namespace: %v", err)
	}
	project, err := projectRepository.Create(ctx, projectrepo.CreateInput{NamespaceID: namespace.ID, Name: "Gity", PathKey: "gity", Visibility: "private"}, namespace)
	if err != nil {
		t.Fatalf("create project: %v", err)
	}
	if err := gitRunner.InitBare(ctx, project.FullPath+".git", "main"); err != nil {
		t.Fatalf("init bare repo: %v", err)
	}
	if err := gitRunner.CreateFileCommit(ctx, project.FullPath+".git", gitexec.CreateFileCommitInput{
		BranchName:  "main",
		FilePath:    "README.md",
		Content:     "hello source archive\n",
		Message:     "Add README",
		AuthorName:  "Gity Test",
		AuthorEmail: "test@gity.dev",
	}); err != nil {
		t.Fatalf("create source commit: %v", err)
	}

	registration, err := runnerSvc.RegisterProjectRunner(ctx, project.ID, runnerservice.RegisterInput{Name: "linux-amd64", Tags: "go, linux, go"})
	if err != nil {
		t.Fatalf("register runner: %v", err)
	}
	if !strings.HasPrefix(registration.Token, "grt_") || registration.Runner.Name != "linux-amd64" || registration.Runner.Tags != "go,linux" {
		t.Fatalf("unexpected registration: %+v", registration)
	}

	runners, err := runnerSvc.ListProjectRunners(ctx, project.ID)
	if err != nil {
		t.Fatalf("list runners: %v", err)
	}
	if len(runners) != 1 || runners[0].ID != registration.Runner.ID || runners[0].Active != true {
		t.Fatalf("unexpected runners: %+v", runners)
	}

	heartbeat, err := runnerSvc.Heartbeat(ctx, registration.Token)
	if err != nil {
		t.Fatalf("heartbeat: %v", err)
	}
	if heartbeat.Status != projectrunnerrepo.StatusOnline || heartbeat.LastContactAt == nil {
		t.Fatalf("unexpected heartbeat view: %+v", heartbeat)
	}

	created, err := jobSvc.EnqueueProjectJob(ctx, project.ID, jobservice.CreateInput{Payload: `{"runner":"test"}`})
	if err != nil {
		t.Fatalf("enqueue job: %v", err)
	}
	noopClaim, err := runnerSvc.ClaimJob(ctx, registration.Token, time.Minute)
	if err != nil {
		t.Fatalf("claim noop job: %v", err)
	}
	if noopClaim.Claimed {
		t.Fatalf("runner should not claim internal noop jobs: %+v", noopClaim)
	}
	if _, err := jobSvc.RunNext(ctx, "worker-a", time.Minute); err != nil {
		t.Fatalf("run internal noop job: %v", err)
	}
	completedNoop, err := jobSvc.GetProjectJob(ctx, project.ID, created.ID)
	if err != nil {
		t.Fatalf("get internal noop job: %v", err)
	}
	if completedNoop.Status != projectjobrepo.StatusSucceeded {
		t.Fatalf("unexpected internal noop job status: %+v", completedNoop)
	}

	created, err = jobSvc.EnqueueProjectJob(ctx, project.ID, jobservice.CreateInput{
		Kind:    jobservice.KindScript,
		Payload: fmt.Sprintf(`{"project_full_path":%q,"ref_name":"main","script":["echo hello"]}`, project.FullPath),
	})
	if err != nil {
		t.Fatalf("enqueue script job: %v", err)
	}
	claim, err := runnerSvc.ClaimJob(ctx, registration.Token, time.Minute)
	if err != nil {
		t.Fatalf("claim job: %v", err)
	}
	if !claim.Claimed || claim.Job.ID != created.ID || claim.Job.Status != projectjobrepo.StatusRunning {
		t.Fatalf("unexpected claim: %+v", claim)
	}
	trace, err := runnerSvc.AppendTrace(ctx, registration.Token, created.ID, runnerservice.AppendTraceInput{Output: "hello\n", DurationMillis: 10})
	if err != nil {
		t.Fatalf("append trace: %v", err)
	}
	if trace.Trace != "hello" || trace.DurationMillis != 10 {
		t.Fatalf("unexpected runner trace: %+v", trace)
	}
	archive, err := runnerSvc.DownloadSourceArchive(ctx, registration.Token, created.ID)
	if err != nil {
		t.Fatalf("download source archive: %v", err)
	}
	if archive.Encoding != "base64" || archive.FileName == "" {
		t.Fatalf("unexpected source archive metadata: %+v", archive)
	}
	content, err := base64.StdEncoding.DecodeString(archive.ContentBase64)
	if err != nil {
		t.Fatalf("decode source archive: %v", err)
	}
	if !zipContainsFile(t, content, "README.md", "hello source archive") {
		t.Fatalf("source archive does not contain expected README")
	}

	completed, err := runnerSvc.CompleteJob(ctx, registration.Token, created.ID, `{"ok":true}`)
	if err != nil {
		t.Fatalf("complete job: %v", err)
	}
	if completed.Status != projectjobrepo.StatusSucceeded || completed.Result == "" {
		t.Fatalf("unexpected completed job: %+v", completed)
	}

	retryable, err := jobSvc.EnqueueProjectJob(ctx, project.ID, jobservice.CreateInput{Kind: jobservice.KindScript, Payload: `{"script":["echo retry"]}`, MaxAttempts: 2})
	if err != nil {
		t.Fatalf("enqueue retryable job: %v", err)
	}
	if _, err := runnerSvc.ClaimJob(ctx, registration.Token, time.Minute); err != nil {
		t.Fatalf("claim retryable job: %v", err)
	}
	failed, err := runnerSvc.FailJob(ctx, registration.Token, retryable.ID, "executor failed", "", time.Millisecond)
	if err != nil {
		t.Fatalf("fail retryable job: %v", err)
	}
	if failed.Status != projectjobrepo.StatusPending || failed.LastError != "executor failed" {
		t.Fatalf("unexpected retryable failure state: %+v", failed)
	}
}

func zipContainsFile(t *testing.T, content []byte, fileName string, contains string) bool {
	t.Helper()
	reader, err := zip.NewReader(bytes.NewReader(content), int64(len(content)))
	if err != nil {
		t.Fatalf("open source archive: %v", err)
	}
	for _, file := range reader.File {
		if file.Name != fileName {
			continue
		}
		rc, err := file.Open()
		if err != nil {
			t.Fatalf("open source archive file: %v", err)
		}
		var buffer bytes.Buffer
		if _, err := buffer.ReadFrom(rc); err != nil {
			_ = rc.Close()
			t.Fatalf("read source archive file: %v", err)
		}
		if err := rc.Close(); err != nil {
			t.Fatalf("close source archive file: %v", err)
		}
		return strings.Contains(buffer.String(), contains)
	}
	return false
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
