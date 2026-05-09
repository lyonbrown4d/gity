package pipeline_test

import (
	"context"
	"fmt"
	"log/slog"
	"path/filepath"
	"testing"

	jobservice "github.com/DaiYuANg/gity/internal/application/job"
	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
	"github.com/DaiYuANg/gity/internal/config"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_exec"
	"github.com/DaiYuANg/gity/internal/infrastructure/git_repo"
	"github.com/DaiYuANg/gity/internal/infrastructure/persistence/core"
	namespacerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/namespace"
	projectrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project"
	projectjobrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_job"
	projectpipelinerepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_pipeline"
	projectpipelinejobrepo "github.com/DaiYuANg/gity/internal/infrastructure/persistence/project_pipeline_job"
	"github.com/DaiYuANg/gity/internal/testutil"
	"github.com/arcgolabs/dbx"
	sqliteDialect "github.com/arcgolabs/dbx/dialect/sqlite"
	_ "modernc.org/sqlite"
)

func pipelineConfig() string {
	return `
pipeline {
  name = "release"
}

stage test {
  timeout_seconds = 300
  run {
    shell("go test ./...")
  }
}

stage build {
  needs = [test]
  artifacts = ["dist/**"]
  run {
    exec("go", "build", "./cmd/server")
  }
}
`
}

func parallelPipelineConfig() string {
	return `
pipeline {
  name = "parallel"
}

stage test {
  run {
    shell("go test ./...")
  }
}

stage lint {
  run {
    shell("golangci-lint run ./...")
  }
}
`
}

type pipelineTestEnv struct {
	ProjectID       int64
	ProjectFullPath string
	DefaultBranch   string
	GitRunner       *gitexec.Runner
	GitRepository   *gitrepo.Service
	JobService      *jobservice.Service
	Service         *pipelineservice.Service
}

func setupPipelineTest(ctx context.Context, t *testing.T) pipelineTestEnv {
	t.Helper()
	return setupPipelineEnv(ctx, t, false)
}

func setupGitPipelineTest(ctx context.Context, t *testing.T) pipelineTestEnv {
	t.Helper()
	return setupPipelineEnv(ctx, t, true)
}

func setupPipelineEnv(ctx context.Context, t *testing.T, withGit bool) pipelineTestEnv {
	t.Helper()

	db := openTestDB(t)
	testutil.CleanupClose(t, "db", db)
	testutil.RequireNoError(t, core.EnsureSchema(ctx, db), "ensure schema")

	namespaceRepository := testutil.Must(namespacerepo.NewRepository(db))
	projectRepository := testutil.Must(projectrepo.NewRepository(db))
	jobRepository := testutil.Must(projectjobrepo.NewRepository(db))
	pipelineRepository := testutil.Must(projectpipelinerepo.NewRepository(db))
	pipelineJobRepository := testutil.Must(projectpipelinejobrepo.NewRepository(db))
	gitRunner, gitRepository := pipelineGitServices(t, withGit)
	jobSvc := jobservice.NewService(slog.Default(), projectRepository, jobRepository, nil, nil, nil)
	service := pipelineservice.NewService(projectRepository, pipelineRepository, pipelineJobRepository, jobSvc, jobRepository, gitRepository)
	namespace := testutil.Must(namespaceRepository.Create(ctx, namespacerepo.CreateInput{Kind: "group", Name: "Core Team", PathKey: "core-team"}))
	projectInput := projectrepo.CreateInput{NamespaceID: namespace.ID, Name: "Gity", PathKey: "gity", Visibility: "private"}
	if withGit {
		projectInput.DefaultBranch = "main"
	}
	project := testutil.Must(projectRepository.Create(ctx, projectInput, namespace))

	return pipelineTestEnv{
		ProjectID:       project.ID,
		ProjectFullPath: project.FullPath,
		DefaultBranch:   project.DefaultBranch,
		GitRunner:       gitRunner,
		GitRepository:   gitRepository,
		JobService:      jobSvc,
		Service:         service,
	}
}

func pipelineGitServices(t *testing.T, enabled bool) (*gitexec.Runner, *gitrepo.Service) {
	t.Helper()
	if !enabled {
		return nil, nil
	}
	repoRoot := t.TempDir()
	settings := config.Settings{Git: config.GitSettings{Bin: "git", RepoRoot: repoRoot}}
	return gitexec.NewRunner(settings), gitrepo.NewService(settings)
}

func createTestPipeline(ctx context.Context, t *testing.T, env pipelineTestEnv, configContent string) pipelineservice.PipelineView {
	t.Helper()
	return testutil.Must(env.Service.CreatePipeline(ctx, env.ProjectID, pipelineservice.CreatePipelineInput{
		Source:        "api",
		RefName:       "main",
		ConfigSource:  ".gity-ci.plano",
		ConfigContent: configContent,
	}))
}

func openTestDB(t *testing.T) *dbx.DB {
	t.Helper()
	dbPath := filepath.Join(t.TempDir(), "gity-pipeline-test.db")
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
