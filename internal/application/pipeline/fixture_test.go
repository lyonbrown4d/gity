package pipeline_test

import (
	"context"
	"fmt"
	"log/slog"
	"path/filepath"
	"testing"

	jobservice "github.com/DaiYuANg/gity/internal/application/job"
	pipelineservice "github.com/DaiYuANg/gity/internal/application/pipeline"
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
	ProjectID  int64
	JobService *jobservice.Service
	Service    *pipelineservice.Service
}

func setupPipelineTest(ctx context.Context, t *testing.T) pipelineTestEnv {
	t.Helper()
	db := openTestDB(t)
	testutil.CleanupClose(t, "db", db)
	if schemaErr := core.EnsureSchema(ctx, db); schemaErr != nil {
		t.Fatalf("ensure schema: %v", schemaErr)
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
	pipelineRepository, err := projectpipelinerepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new pipeline repo: %v", err)
	}
	pipelineJobRepository, err := projectpipelinejobrepo.NewRepository(db)
	if err != nil {
		t.Fatalf("new pipeline job repo: %v", err)
	}
	jobSvc := jobservice.NewService(slog.Default(), projectRepository, jobRepository, nil, nil, nil)
	service := pipelineservice.NewService(projectRepository, pipelineRepository, pipelineJobRepository, jobSvc, jobRepository, nil)
	namespace, err := namespaceRepository.Create(ctx, namespacerepo.CreateInput{Kind: "group", Name: "Core Team", PathKey: "core-team"})
	if err != nil {
		t.Fatalf("create namespace: %v", err)
	}
	project, err := projectRepository.Create(ctx, projectrepo.CreateInput{NamespaceID: namespace.ID, Name: "Gity", PathKey: "gity", Visibility: "private"}, namespace)
	if err != nil {
		t.Fatalf("create project: %v", err)
	}
	return pipelineTestEnv{ProjectID: project.ID, JobService: jobSvc, Service: service}
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
